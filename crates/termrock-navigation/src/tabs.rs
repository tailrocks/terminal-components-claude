use core::fmt;
use core::marker::PhantomData;
use core::time::Duration;

use ratatui_core::buffer::CellWidth;
use ratatui_core::layout::{Position, Rect};

use super::{Acc, PartStyle, SlotFn, cell_at, first_row, paint_pressed_bracket};
use crate::event::{Chord, KeyCode, KeyModifiers};
use crate::focus::Focusability;
use crate::id::{Id, ItemKey, Part, PartRef};
use crate::intent::{Intent, Phase};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::{Response, StateFlags};
use crate::theme::{Family, GlyphRole, Role, Slot, StyleDefaults, StylePatch, Variant};
use crate::ui::{Cx, FrameRead, LayoutFacts, Ui};
use termrock_collections::collection::{
    ByIndex, CollectionCore, DefaultRow, KeyFn, Reconcile, Reconciliation,
};
use termrock_runtime::empty::Status;
use termrock_runtime::rowui::{RowFn, RowUi};

/// Keyboard aliases used for adjacent tab navigation.
///
/// The default `None` profile preserves the existing arrow and lowercase
/// `h`/`l` bindings. This opt-in profile changes only the Prev/Next keyboard
/// descriptors; it does not change layout or pointer behavior. The profiles
/// cover adjacent navigation only and do not add Home/End or `g`/`G` actions.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabNavProfile {
    /// Use Left/Right arrows, including Shift+Left/Right.
    Desktop,
    /// Use Desktop arrows plus `h`/`H` for Prev and `l`/`L` for Next.
    Vim,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabsAction {
    Activated(ItemKey),
    Close(ItemKey),
    New,
}

/// Controls when an eligible tab's close affordance is painted and registered.
///
/// Both modes reserve the same close-target geometry. In
/// [`Hover`](Self::Hover) mode, the target is a live hit only while its tab is
/// active or focused, its tab or close part is hovered, or its close part is
/// pressed.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TabCloseReveal {
    /// Keep every eligible close affordance visible. This preserves the
    /// historical default.
    #[default]
    Always,
    /// Reveal an eligible close affordance only while its tab is in use.
    Hover,
}

/// Foreground policy for the active tab's rule.
///
/// The policy changes only the active rule's foreground. The default Tabs
/// rendering path is unchanged unless a policy is selected. This enum is
/// non-exhaustive so additional rule policies can be added without breaking
/// downstream matches.
#[non_exhaustive]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TabRule {
    /// Resolve the active rule with the theme's Accent role.
    Accent,
    /// Resolve the active rule with the theme's BorderStrong role.
    Neutral,
    /// Use Accent while the Tabs context is not hidden; otherwise use
    /// BorderStrong.
    ///
    /// Without an explicit host override, hidden means that a native runtime
    /// layer covers the current drawing context.
    Auto,
    /// Use Accent only while this Tabs owner has focus and its context is not
    /// hidden; otherwise use BorderStrong.
    ///
    /// Focus is presentation data, not input permission. Without an explicit
    /// host override, hidden means that a native runtime layer covers the
    /// current drawing context.
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

/// Independent status facts for one tab item.
///
/// When any fact is set, Tabs reserves a two-cell [`Part::ICON`] lane after
/// the row content and before its close affordance. If several facts are set,
/// the visible mark uses this priority: busy, error, dirty, then done. The
/// type keeps its storage private so callers can use builders without taking
/// a dependency on its exact representation.
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

/// Configuration errors for Tabs navigation and replacement New/Close groups.
///
/// A group may contain only its corresponding command. Action identities must
/// be unique across the effective Tabs table, and declared chords must not
/// collide under [`Chord::effective_eq`]. The default New or Close group is
/// omitted from validation when that same group is replaced.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TabsKeyBindingsError {
    /// A binding in the New group had a command other than [`TabsCmd::New`].
    NewGroupCommand { index: usize, found: TabsCmd },
    /// A binding in the Close group had a command other than [`TabsCmd::Close`].
    CloseGroupCommand { index: usize, found: TabsCmd },
    /// Two effective bindings reused one action identity.
    DuplicateAction { action: crate::ActionKey },
    /// Two effective bindings declared chords that match the same input.
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

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct TabsState {
    core: CollectionCore,
    active: Option<ItemKey>,
    first: Option<ItemKey>,
    first_index: usize,
    // Defer the live CLOSE target while its hidden cell belongs to a TAB press.
    close_reveal_press: Option<ItemKey>,
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
        self.active = Some(key);
        self.core.set_cursor(index, key);
    }
}

impl Reconcile for TabsState {
    fn reconcile(&mut self, len: usize, key: impl Fn(usize) -> ItemKey) -> Reconciliation {
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
        r
    }

    fn invalidate(&mut self) {
        self.core.invalidate();
        self.close_reveal_press = None;
    }
}

/// Committed viewport counts supplied to a pure tabs layout pass.
///
/// This receipt is explicit because planning must not inspect a previously
/// painted frame. Pass the receipt returned by the preceding committed layout
/// when preserving the warm-frame overflow reservation rule.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct TabWindowReceipt {
    pub viewport_len: usize,
    pub content_len: usize,
}

/// One immutable semantic row allocation for a [`TabLayout`].
#[derive(Clone)]
pub struct TabRowLayout {
    pub key: ItemKey,
    pub index: usize,
    pub logical_content: Rect,
    pub visible_content: Rect,
    pub logical_tab: Rect,
    pub visible_tab: Rect,
    /// Allocated close-target geometry for an eligible item. An explicitly
    /// widened target may overlap label or mark cells. In
    /// [`TabCloseReveal::Hover`] mode this remains present while the affordance
    /// is hidden.
    pub logical_close: Option<Rect>,
    /// Clipped allocation for an eligible close target, not a promise that
    /// the current frame registered a live hit. The committed UI hit registry
    /// is authoritative for current interaction visibility.
    pub visible_close: Option<Rect>,
    logical_mark: Option<Rect>,
    visible_mark: Option<Rect>,
    mark: Option<TabItemMark>,
    close_enabled: bool,
    label_segments: Vec<String>,
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

/// One item snapshot owned by a fresh planned default layout.
#[derive(Clone, PartialEq, Eq, Debug)]
struct OwnedTabMeasurement {
    key: ItemKey,
    mark: Option<TabItemMark>,
    close_enabled: bool,
    label_segments: Vec<String>,
}

/// One row's exact native allocation, before choosing how to retain it.
///
/// Terminal probes keep only the last record; the committed layout sink turns
/// records into public rows and copies the captured display segments once.
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
    #[cfg(test)]
    fn from_tab_row(row: &TabRowLayout) -> Self {
        Self {
            key: row.key,
            index: row.index,
            logical_content: row.logical_content,
            visible_content: row.visible_content,
            logical_tab: row.logical_tab,
            visible_tab: row.visible_tab,
            logical_close: row.logical_close,
            visible_close: row.visible_close,
            logical_mark: row.logical_mark,
            visible_mark: row.visible_mark,
            mark: row.mark,
            close_enabled: row.close_enabled,
            label_extent: row.label_extent,
        }
    }

    fn into_tab_row(self, label_segments: Vec<String>) -> TabRowLayout {
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
    new_area: Option<Rect>,
    status_area: Option<Rect>,
    hidden_right: usize,
    viewport_len: usize,
    last_row: Option<DefaultRowGeometry>,
}

fn max_rows_for_width_budget(available: u16) -> usize {
    // Compute A + 1 in a wider type; `available` can be u16::MAX.
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

/// Pure layout result for default display rows.
///
/// Layout owns every formatted default-row segment and all tab allocations.
/// Drawing a layout never invokes item Display or mark callbacks and never
/// scans rendered buffer cells to recover label extents.
#[derive(Clone)]
pub struct TabLayout {
    id: Id,
    area: Rect,
    clip: Rect,
    row_area: Rect,
    used_area: Rect,
    rule_area: Option<Rect>,
    rows: Vec<TabRowLayout>,
    // Some holds every item in a nonempty planned layout. None is the
    // callback-free empty-area plan. Excluded from historical public Eq/Debug.
    owned_items: Option<Vec<OwnedTabMeasurement>>,
    // Private bound: the earliest suffix whose final allocated tab is visible.
    terminal_first_index: Option<usize>,
    first_index: usize,
    active: Option<ItemKey>,
    cursor: Option<ItemKey>,
    first: Option<ItemKey>,
    first_tab_x: u16,
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
    new_area: Option<Rect>,
    status_area: Option<Rect>,
    item_marks_enabled: bool,
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
            && self.first_index == other.first_index
            && self.active == other.active
            && self.cursor == other.cursor
            && self.first == other.first
            && self.first_tab_x == other.first_tab_x
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
            && self.new_area == other.new_area
            && self.status_area == other.status_area
            && self.item_marks_enabled == other.item_marks_enabled
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

    pub const fn used_area(&self) -> Rect {
        self.used_area
    }

    /// Return logical and clipped allocated bounds for a tabs part.
    ///
    /// A close bound is potential geometry for an eligible item. With
    /// [`TabCloseReveal::Hover`], the close affordance may be hidden and have
    /// no live hit target; query the committed UI hit registry to determine
    /// current interaction visibility.
    pub fn part_bounds(&self, part: PartRef) -> Option<(Rect, Rect)> {
        match part.part {
            Part::TAB => self
                .rows
                .iter()
                .find(|row| Some(row.key) == part.item)
                .map(|row| (row.logical_tab, row.visible_tab)),
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
                && self.terminal_first_index.is_none();
        };
        if self.area.is_empty()
            || owned_items.len() != self.content_len
            || self
                .terminal_first_index
                .is_some_and(|first| first >= self.content_len)
        {
            return false;
        }
        self.rows.iter().all(|row| {
            owned_items.get(row.index).is_some_and(|item| {
                item.key == row.key
                    && item.mark == row.mark
                    && item.close_enabled == row.close_enabled
            })
        })
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
    item_marks: Option<fn(&T) -> TabMarks>,
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
            item_marks: None,
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

    /// Select an adjacent-tab navigation profile.
    ///
    /// `None` preserves the current default table exactly. A profile changes
    /// only Prev/Next descriptors; Activate, digit, New, Close, pointer,
    /// overflow, layout, and wrapping semantics stay unchanged. The selected
    /// descriptors are used for both binding publication and dispatch.
    ///
    /// # Errors
    ///
    /// Returns an error when a current custom New or Close binding reuses an
    /// action identity or effective chord with the selected profile or another
    /// effective binding.
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
            item_marks: self.item_marks,
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
            item_marks: self.item_marks,
            ov: self.ov,
            _t: PhantomData,
        }
    }

    #[must_use]
    pub fn allow_new(mut self, yes: bool) -> Self {
        self.allow_new = yes;
        self
    }

    /// Enable the built-in `1`–`9` shortcuts for selecting tabs by position.
    ///
    /// The default is `true`. Disabling digit jumps removes those actions from
    /// the published key tables while leaving tab layout and pointer behavior
    /// unchanged.
    #[must_use]
    pub fn digit_jump(mut self, yes: bool) -> Self {
        self.digit_jump = yes;
        self
    }

    /// Wrap the existing previous/next tab commands at the ends of the list.
    ///
    /// The default is `false`, which preserves the current endpoint behavior.
    /// When enabled, only the existing previous/next key actions cycle between
    /// the first and last tabs; pointer, overflow, and wheel interaction is
    /// unchanged.
    #[must_use]
    pub fn wrap(mut self, yes: bool) -> Self {
        self.wrap = yes;
        self
    }

    /// Replace the keyboard bindings for [`TabsCmd::New`].
    ///
    /// The default `None` keeps the built-in New binding in the base
    /// [`Bindings`] table. `Some(&[])` explicitly removes New keyboard
    /// bindings while leaving the pointer New part governed by
    /// [`Self::allow_new`]. A custom static table is published with its
    /// original action, chord, label, priority, and visibility metadata.
    ///
    /// The complete potential command table is validated, even when New is
    /// currently disabled or this Tabs instance is passive. Use
    /// [`Self::new_key_table`] to publish the supplemental table when painting
    /// manually alongside [`Bindings::bindings`].
    ///
    /// # Errors
    ///
    /// Returns an error when a binding uses a command other than
    /// [`TabsCmd::New`], reuses an effective action identity, or has a chord
    /// that conflicts with another effective binding.
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

    /// Replace the keyboard bindings for [`TabsCmd::Close`].
    ///
    /// The default `None` keeps the built-in Close bindings in the base
    /// [`Bindings`] table. `Some(&[])` explicitly removes Close keyboard
    /// bindings without changing close-target eligibility, pointer clicks, or
    /// Middle behavior. A custom static table is published with its original
    /// action, chord, label, priority, and visibility metadata.
    ///
    /// The complete potential command table is validated, even when no item
    /// is currently closable or this Tabs instance is passive. Use
    /// [`Self::close_key_table`] to publish the supplemental table when
    /// painting manually alongside [`Bindings::bindings`].
    ///
    /// # Errors
    ///
    /// Returns an error when a binding uses a command other than
    /// [`TabsCmd::Close`], reuses an effective action identity, or has a chord
    /// that conflicts with another effective binding.
    pub fn try_close_key_bindings(
        mut self,
        bindings: Option<&'static [Binding<TabsCmd>]>,
    ) -> Result<Self, TabsKeyBindingsError> {
        validate_tabs_key_bindings(self.nav_profile, self.new_key_bindings, bindings)?;
        self.close_key_bindings = bindings;
        Ok(self)
    }

    /// Close the currently closable tab under a middle-button press.
    ///
    /// The default is `true`. This emits [`TabsAction::Close`] for the
    /// pointed stable item key without activating or moving the tab cursor.
    /// Close eligibility remains controlled by [`Self::closable`] and
    /// [`Self::closable_keys`].
    #[must_use]
    pub fn middle_click_closes(mut self, yes: bool) -> Self {
        self.middle_click_closes = yes;
        self
    }

    /// Set close eligibility by stable item key.
    ///
    /// When this policy is present, only current items with a listed key get a
    /// close affordance. It overrides [`Self::closable`], including when the
    /// slice is empty. When this method is not called, [`Self::closable`]
    /// keeps its existing all-items behavior.
    #[must_use]
    pub fn closable_keys(mut self, keys: &'a [ItemKey]) -> Self {
        self.closable_keys = Some(keys);
        self
    }

    /// Choose when an eligible close affordance is visible and interactive.
    ///
    /// The default is [`TabCloseReveal::Always`]. [`TabCloseReveal::Hover`]
    /// keeps each eligible close target reserved so tab geometry does not
    /// shift, but registers it only while its tab is active or focused, its
    /// tab or close part is hovered, or its close part is pressed.
    #[must_use]
    pub fn close_reveal(mut self, reveal: TabCloseReveal) -> Self {
        self.close_reveal = reveal;
        self
    }

    /// Widen the close hit target around its existing close glyph.
    ///
    /// The default is `None`, which preserves the current one-cell target.
    /// Explicit widths follow the centered one-cell-glyph plate calculation:
    /// even requests round down to the nearest odd width, and zero means one
    /// cell. The target is clamped within each tab and does not change tab
    /// sizing or the reserved close-label space.
    #[must_use]
    pub fn close_target_w(mut self, cells: u16) -> Self {
        self.close_target_w = Some(cells);
        self
    }

    /// Use danger semantic colors while the close target is hovered or
    /// pressed.
    ///
    /// The default is `false`, preserving the theme's existing CLOSE recipe.
    /// This changes only the close target's hovered and pressed presentation.
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

    /// Choose the foreground role for the active rule.
    ///
    /// The default is unset, which preserves the existing theme-resolved
    /// active rule. An explicit RULE foreground in this Tabs instance takes
    /// precedence over the selected policy.
    #[must_use]
    pub fn rule(mut self, rule: TabRule) -> Self {
        self.rule = Some(rule);
        self
    }

    /// Override native-layer coverage for the dynamic active-rule policies.
    ///
    /// Without this setting, [`TabRule::Auto`] and [`TabRule::FocusOnly`]
    /// consult the current frame's native layer stack. Set this to `true` when
    /// a host-owned overlay or popup hides the Tabs context without opening a
    /// native layer; set it to `false` to explicitly present the rule as
    /// uncovered. This changes only active-rule presentation, never focus,
    /// hit registration, or input eligibility. It has no effect on the fixed
    /// [`TabRule::Accent`] and [`TabRule::Neutral`] policies.
    #[must_use]
    pub fn rule_focus_hidden(mut self, hidden: bool) -> Self {
        self.rule_focus_hidden = Some(hidden);
        self
    }

    /// Keep the tab strip out of the automatic focus order and keyboard map.
    ///
    /// Visible tab, close, new and overflow parts remain interactive. The
    /// default is `false`, which preserves the ordinary focusable strip.
    #[must_use]
    pub fn passive(mut self, passive: bool) -> Self {
        self.passive = passive;
        self
    }

    /// Choose whether a passive strip presents its cursor row as focused.
    ///
    /// This only affects presentation while [`Self::passive`] is `true`; it
    /// never grants focus or keyboard input. The default is `false`.
    #[must_use]
    pub fn passive_focused(mut self, focused: bool) -> Self {
        self.passive_focused = focused;
        self
    }

    /// Map each item to independent optional status marks.
    ///
    /// The default is unconfigured and does not reserve per-item space. A
    /// configured mapper that returns clean marks also reserves no space.
    /// Nonempty marks reserve two cells after the row content and before an
    /// eligible close affordance. The mapper's result is copied into a
    /// [`TabLayout`] when planning; rebuild that plan after changing item data
    /// or the mapper to display the new facts.
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

    /// Supplemental New bindings for the current keyboard configuration.
    ///
    /// The built-in New command remains in the unchanged base table returned
    /// by [`Bindings::bindings`] while no custom New table is configured. This
    /// accessor returns only the extra table to publish when a custom table
    /// replaces that default, and returns an empty slice when New is gated or
    /// the Tabs owner is passive.
    pub fn new_key_table(&self) -> &'static [Binding<TabsCmd>] {
        self.effective_key_groups().new
    }

    /// Supplemental Close bindings for the current keyboard configuration.
    ///
    /// The built-in Close commands remain in the unchanged base table
    /// returned by [`Bindings::bindings`] while no custom Close table is
    /// configured. This accessor returns only the extra table to publish when
    /// a custom table replaces those defaults, and returns an empty slice when
    /// Close is gated or the Tabs owner is passive.
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
            // Keep the original static table and descriptor order byte-for-byte
            // when no navigation profile or optional group is configured.
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

    // Validate every command that could become active, regardless of current
    // allow_new, close eligibility, or passive presentation. A configured
    // override replaces only its own default group in this potential union.
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

    pub fn update(&self, cx: &mut Cx<'_>, st: &mut TabsState, items: &[T]) -> Response<TabsAction> {
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
        if st.active.is_none() {
            st.active = st.core.cursor();
        }
        if st.first.is_none() && len > 0 {
            st.first = Some(self.key_at(items, 0));
            st.first_index = 0;
        }
        let mut acc = Acc::<TabsAction>::new();
        let key_groups = self.effective_key_groups();
        for it in cx.intents(self.id) {
            match it {
                Intent::Cancel | Intent::FocusOut { .. } | Intent::Layer(_) => {
                    st.close_reveal_press = None;
                }
                Intent::Binding(action) => {
                    if self.passive {
                        acc.consumed();
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
                        (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(0))) => {
                            let i = st.first_index.saturating_sub(1);
                            st.first_index = i;
                            st.first = (len > 0).then(|| self.key_at(items, i));
                            acc.changed();
                        }
                        (Phase::Click, Part::OVERFLOW, Some(ItemKey::Index(_))) => {
                            let i = st.first_index.saturating_add(1).min(len.saturating_sub(1));
                            st.first_index = i;
                            st.first = (len > 0).then(|| self.key_at(items, i));
                            acc.changed();
                        }
                        _ => acc.consumed(),
                    }
                }
                _ => {}
            }
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
        if let Some(l) = cx.layout(self.id) {
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

    #[expect(
        clippy::too_many_lines,
        reason = "one shared paint pass consumes either legacy rows or an owned semantic layout"
    )]
    fn draw_impl(&self, ui: &mut Ui<'_>, area: Rect, source: TabsRenderSource<'_, T>) -> Rect {
        if area.is_empty() {
            return area;
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
            {
                let mut r = RowUi::new(ui, id, Family::TABS, Variant::DEFAULT, flags, key, content);
                match row_source {
                    TabsRowSource::Legacy(item) => self.row.row(item, &mut r),
                    TabsRowSource::Planned(row) => {
                        r.label_fmt(format_args!("{}", DisplaySegments(&row.label_segments)));
                    }
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
            let ts = ov.style(ui, id, Family::TABS, Variant::DEFAULT, Part::TAB, flags);
            ui.paint_style(tab, ts.style);

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
                    // A previous visible frame may have left a custom glyph or
                    // close-state style in this reserved glyph lane. Restore
                    // that lane only; row text and marks have already been
                    // repainted, and may overlap a widened target.
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
            first_index,
            active: state.active,
            cursor: state.core.cursor(),
            first: state.first,
            first_tab_x: area.x,
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
            new_area: None,
            status_area: None,
            item_marks_enabled: self.item_marks.is_some(),
            right_limit: area.x,
            hidden_right: len.saturating_sub(first_index),
            logical_area: area,
            visible_area: area.intersection(clip),
            viewport_len: 0,
            content_len: len,
            overflow_reserved,
        }
    }

    /// Resolve default display rows without painting or reading a buffer.
    ///
    /// Every fresh nonempty plan formats each item once and captures its key,
    /// visible mark, close eligibility, and original Display write segments,
    /// including items that are not allocated in the visible window. Rebuild
    /// after item content changes; there is no persistent same-key text cache.
    /// Empty-area plans preserve the original callback/key lookup behavior.
    ///
    /// `previous` is the caller-owned receipt from the last committed draw.
    /// It preserves the existing overflow reservation behavior across cold
    /// and warm frames while keeping measurement independent of `Ui`.
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
            // Match the original empty-area key lookup exactly, including its
            // optional hinted lookup and repeated hinted key call on fallback.
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
                OwnedTabMeasurement {
                    key,
                    mark,
                    close_enabled,
                    label_segments: writer.0,
                }
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

        // Preserve the exact current terminal predicate and ascending candidate
        // order while skipping only positive starts that cannot emit enough
        // rows to reach the final item. Candidate zero has different lane
        // reservations, so it is probed separately.
        let mut extent_scratch = Vec::new();
        let terminal_first_index =
            self.find_terminal_first_index(area, clip, &owned_items, previous, &mut extent_scratch);

        let mut layout = self.layout_default_from_owned(
            area,
            clip,
            state,
            &owned_items,
            first_index,
            previous,
            &mut extent_scratch,
        );
        layout.owned_items = Some(owned_items);
        layout.terminal_first_index = terminal_first_index;
        layout
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

        // Start zero has different left/right reservations; preserve its
        // original first place in the exhaustive search explicitly.
        let start_zero =
            self.walk_default_owned(area, clip, items, 0, previous, extent_scratch, |_| {});
        if reaches_final(start_zero) {
            return Some(0);
        }

        // Every positive start has the same four-cell left and right lanes.
        // At least three cells are consumed by each emitted row, and each
        // non-final row advances x by at least four. Thus K bounds the number
        // of rows any positive-start walk can emit. This header is shared with
        // the walker, so the bound follows the actual status/New/reservation
        // geometry rather than a separate width approximation.
        let positive_header = self.default_row_header(area, 1, previous);
        let max_rows = max_rows_for_width_budget(positive_header.available_width());
        positive_terminal_candidate_range(len, max_rows).find(|&candidate_first| {
            let walk = self.walk_default_owned(
                area,
                clip,
                items,
                candidate_first,
                previous,
                extent_scratch,
                |_| {},
            );
            reaches_final(walk)
        })
    }

    fn default_row_header(
        &self,
        area: Rect,
        first_index: usize,
        previous: Option<TabWindowReceipt>,
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
        let right_w: u16 = if overflow_reserved || first_index > 0 {
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

        DefaultRowHeader {
            row_area,
            used_area,
            rule_area,
            first_tab_x,
            right_limit,
            left_overflow,
            status_w,
            new_w,
        }
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
        mut emit: impl FnMut(DefaultRowGeometry),
    ) -> DefaultRowWalk {
        debug_assert!(
            !area.is_empty(),
            "empty geometry is handled before the walk"
        );
        let len = items.len();
        let first_index = first_index.min(len);
        let header = self.default_row_header(area, first_index, previous);
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
                    .saturating_sub(if mark.is_some() { close_w } else { 0 }),
                height: 1,
            };
            let label_extent = semantic_label_extent_with_scratch(
                &item.label_segments,
                content,
                clip,
                extent_scratch,
            )
            .max(1);
            let tab_w = 1u16
                .saturating_add(label_extent)
                .saturating_add(mark_width)
                .saturating_add(1)
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
        let right_overflow = (hidden_right > 0).then(|| Rect {
            x: row_area
                .right()
                .saturating_sub(status_w)
                .saturating_sub(new_w)
                .saturating_sub(4)
                .max(row_area.x),
            y: row_area.y,
            width: 4.min(row_area.width),
            height: 1,
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
            new_area,
            status_area,
            hidden_right,
            viewport_len,
            last_row,
        }
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
                first_index,
                active: state.active,
                cursor: state.core.cursor(),
                first: state.first,
                first_tab_x: area.x,
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
                new_area: None,
                status_area: None,
                item_marks_enabled: self.item_marks.is_some(),
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
        let walk = self.walk_default_owned(
            area,
            clip,
            items,
            first_index,
            previous,
            extent_scratch,
            |row| {
                rows.push(row.into_tab_row(items[row.index].label_segments.clone()));
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
            first_index,
            active: state.active,
            cursor: state.core.cursor(),
            first: state.first,
            first_tab_x: walk.first_tab_x,
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
            new_area: walk.new_area,
            status_area: walk.status_area,
            item_marks_enabled: self.item_marks.is_some(),
            right_limit: walk.right_limit,
            hidden_right: walk.hidden_right,
            logical_area: walk.used_area,
            visible_area: walk.used_area.intersection(clip),
            viewport_len: walk.viewport_len,
            content_len: len,
            overflow_reserved,
        }
    }

    /// Draw a previously resolved default-row layout.
    ///
    /// Rebuild the plan after any state or data change. Returns `Err` when
    /// component identity or rendering configuration differs from planning.
    pub fn draw_layout(
        &self,
        ui: &mut Ui<'_>,
        layout: &TabLayout,
    ) -> Result<Rect, TabLayoutMismatch> {
        if !layout.owned_snapshot_is_consistent()
            || layout.id != self.id
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
        {
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

// Preserve the allocating pre-v3 implementation as an independent test oracle.
#[cfg(test)]
fn semantic_label_extent(segments: &[String], content: Rect, clip: Rect) -> u16 {
    // `RowUi::label_fmt` writes each Display segment independently. Each write
    // clips from the current logical writer position, while the painter starts
    // at the clipped left edge. Before the writer reaches that edge, successive
    // segments can therefore overlap and replace cells. Model only those
    // visible semantic cells; do not render into a scratch Buffer or inspect a
    // Ui during measurement.
    let visible = content.intersection(clip);
    if visible.is_empty() {
        return 0;
    }

    let mut cells = vec![false; usize::from(visible.width)];
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

// Same semantic walk as the test oracle, using capacity owned by the caller.
fn semantic_label_extent_with_scratch(
    segments: &[String],
    content: Rect,
    clip: Rect,
    cells: &mut Vec<bool>,
) -> u16 {
    // `RowUi::label_fmt` writes each Display segment independently. Each write
    // clips from the current logical writer position, while the painter starts
    // at the clipped left edge. Before the writer reaches that edge, successive
    // segments can therefore overlap and replace cells. Model only those
    // visible semantic cells; do not render into a scratch Buffer or inspect a
    // Ui during measurement.
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

    // With a replacement group, the built-in rows for that group are removed
    // from this base table. Manual publishers must add the corresponding
    // supplemental table returned by `new_key_table` or `close_key_table`.
    fn bindings(&self, _s: BindingState) -> &'static [Binding<TabsCmd>] {
        self.table()
    }
}

#[cfg(test)]
mod tests {
    use core::cell::Cell;

    use ratatui_core::buffer::{Buffer, Cell as BufferCell};
    use ratatui_core::layout::{Position, Rect};
    use ratatui_core::style::Modifier;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::{ColorLevel, Theme};

    const TABS: Id = Id::root("tabs.tests");
    const AREA: Rect = Rect {
        x: 0,
        y: 0,
        width: 12,
        height: 2,
    };

    fn row_text(buf: &Buffer, width: u16) -> String {
        let mut text = String::new();
        for x in 0..width {
            if let Some(cell) = buf.cell(Position::new(x, 0)) {
                text.push_str(cell.symbol());
            }
        }
        text
    }

    fn draw_status(status: Option<Status>) -> Buffer {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = TabsState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let mut tabs = Tabs::new(TABS);
                if let Some(status) = status {
                    tabs = tabs.status(status);
                }
                tabs.draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        buffer
    }

    #[test]
    fn close_targets_the_logical_tab_after_a_reorder() {
        let mut st = TabsState::default();
        let keys = |v: &[u64]| {
            let v: Vec<ItemKey> = v.iter().map(|n| ItemKey::num(*n)).collect();
            v
        };
        let a = keys(&[1, 2, 3]);
        let _ = st.reconcile(3, |i| a[i]);
        st.set_active(1, ItemKey::num(2));
        st.first = Some(ItemKey::num(1));

        let b = keys(&[9, 3, 2, 1]);
        let r = st.reconcile(4, |i| b[i]);
        assert_eq!(r, Reconciliation::Unchanged);
        assert_eq!(st.active(), Some(ItemKey::num(2)));
        assert_eq!(st.cursor(), Some(ItemKey::num(2)));
        assert_eq!(st.first(), Some(ItemKey::num(1)));
        assert_eq!(st.first_index, 3);

        let c = keys(&[9, 3, 1]);
        let _ = st.reconcile(3, |i| c[i]);
        assert_eq!(st.active(), st.cursor());
        assert!(st.active().is_some());
        let mut buf = [0u8; 2];
        assert_eq!(digits(7, &mut buf), "7");
        assert_eq!(digits(42, &mut buf), "42");
        assert_eq!(digits(500, &mut buf), "99");
    }

    #[test]
    fn mono_pressed_brackets_the_reserved_pad_cells() {
        const LABEL: &str = "Full width";
        let items = [LABEL];
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::index(0));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                ui.reference(
                    Some(
                        crate::ReferenceTarget::new(
                            TABS,
                            crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                        )
                        .part(PartRef::item(Part::TAB, ItemKey::index(0))),
                    ),
                    |ui| Tabs::new(TABS).draw(ui, area, &state, &items),
                );
            })
            .commit_presented();

        assert_eq!(row_text(&buffer, AREA.width), "[Full width]");
    }

    #[test]
    fn mono_pressed_closable_tab_keeps_the_close_cell_after_the_bracket() {
        let items = ["tab"];
        let key = ItemKey::index(0);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(ColorLevel::Mono));
        let mut buffer = Buffer::empty(AREA);
        let mut state = TabsState::default();
        state.set_active(0, key);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                let target = crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::PRESSED | crate::ReferenceState::FOCUSED,
                )
                .part(PartRef::item(Part::TAB, key));
                ui.reference(Some(target), |ui| {
                    Tabs::new(TABS)
                        .closable(true)
                        .draw(ui, area, &state, &items);
                });
            })
            .commit_presented();

        assert_eq!(
            buffer.cell(Position::new(4, 0)).map(BufferCell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::PressRight))
        );
        assert_eq!(
            buffer.cell(Position::new(5, 0)).map(BufferCell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::Close))
        );
    }

    #[test]
    fn readiness_lane_is_far_right_conditional_patchable_and_replaceable() {
        assert!(Tabs::<&str>::PARTS.contains(&Part::ICON));
        assert_eq!(draw_status(None), draw_status(Some(Status::Ready)));
        let busy = draw_status(Some(Status::Busy));
        let frame = Theme::junie()
            .design
            .motion
            .spinner_frames
            .first()
            .copied()
            .unwrap_or("");
        assert_eq!(
            busy.cell(Position::new(10, 0))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(frame)
        );

        let patch = [(Part::ICON, StylePatch::new().add(Modifier::UNDERLINED))];
        let state = TabsState::default();
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS)
                    .status(Status::Busy)
                    .patch_part(&patch)
                    .draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        assert!(
            buffer
                .cell(Position::new(10, 0))
                .is_some_and(|cell| cell.modifier.contains(Modifier::UNDERLINED))
        );

        let seen = Cell::new(None);
        let slot = |_ui: &mut Ui<'_>, area: Rect| seen.set(Some(area));
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS)
                    .status(Status::Error)
                    .slot(Part::ICON, &slot)
                    .draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        assert_eq!(seen.get(), Some(Rect::new(10, 0, 1, 1)));
    }

    #[test]
    fn active_rule_comes_only_from_semantic_state() {
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(AREA);
        let state = TabsState::default();
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS).draw(ui, area, &state, &["tab"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(0, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::RuleQuiet))
        );

        let active = TabsState {
            active: Some(ItemKey::index(0)),
            ..TabsState::default()
        };
        runtime
            .draw_scene(AREA, &mut buffer, |ui, area| {
                Tabs::new(TABS).draw(ui, area, &active, &["tab"]);
            })
            .commit_presented();
        assert_eq!(
            buffer
                .cell(Position::new(0, 1))
                .map(ratatui_core::buffer::Cell::symbol),
            Some(Theme::junie().design.glyphs.get(GlyphRole::RuleActive))
        );
    }

    #[derive(Clone, Copy)]
    struct TabsRenderMode {
        semantic: bool,
        close_reveal: TabCloseReveal,
    }

    fn render_semantic_pair(
        mode: TabsRenderMode,
        area: Rect,
        state: &TabsState,
        items: &[&str],
        previous: Option<TabWindowReceipt>,
        level: ColorLevel,
        target: Option<crate::ReferenceTarget>,
    ) -> (
        Buffer,
        Vec<termrock_runtime::hit::Region>,
        Option<Position>,
        TabLayout,
    ) {
        let tabs = Tabs::new(TABS)
            .allow_new(true)
            .closable(true)
            .close_reveal(mode.close_reveal)
            .status(Status::Error);
        let layout = tabs.layout_default(area, area, state, items, previous);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie().downgrade(level));
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, area| {
                ui.reference(target, |ui| {
                    if mode.semantic {
                        tabs.draw_layout(ui, &layout)
                            .expect("same tabs configuration draws its planned layout")
                    } else {
                        tabs.draw(ui, area, state, items)
                    }
                });
            })
            .commit_presented();
        (
            buffer,
            runtime.registry().regions().to_vec(),
            runtime.cursor(),
            layout,
        )
    }

    #[test]
    fn default_semantic_layout_matches_legacy_cells_parts_and_cursor_profiles() {
        let items = ["", "one ", "界e\u{301}", "x\ty", "tail    ", "next"];
        let mut state = TabsState::default();
        state.set_active(2, ItemKey::index(2));
        let targets = [
            None,
            Some(
                crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::FOCUSED | crate::ReferenceState::FOCUS_VISIBLE,
                )
                .part(PartRef::item(Part::TAB, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(TABS, crate::ReferenceState::HOVERED)
                    .part(PartRef::item(Part::TAB, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::FOCUSED | crate::ReferenceState::PRESSED,
                )
                .part(PartRef::item(Part::TAB, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(
                    TABS,
                    crate::ReferenceState::HOVERED | crate::ReferenceState::PRESSED,
                )
                .part(PartRef::item(Part::CLOSE, ItemKey::index(2))),
            ),
            Some(
                crate::ReferenceTarget::new(TABS, crate::ReferenceState::HOVERED)
                    .part(PartRef::item(Part::TAB, ItemKey::index(3))),
            ),
            Some(
                crate::ReferenceTarget::new(TABS, crate::ReferenceState::HOVERED)
                    .part(PartRef::item(Part::CLOSE, ItemKey::index(3))),
            ),
            Some(
                crate::ReferenceTarget::new(TABS, crate::ReferenceState::PRESSED)
                    .part(PartRef::item(Part::CLOSE, ItemKey::index(3))),
            ),
        ];

        for reveal in [TabCloseReveal::Always, TabCloseReveal::Hover] {
            for level in [
                ColorLevel::TrueColor,
                ColorLevel::Ansi256,
                ColorLevel::Ansi16,
                ColorLevel::Mono,
            ] {
                for width in [0, 1, 2, 8, 13, 24, 40] {
                    let area = Rect::new(3, 2, width, 2);
                    for target in targets {
                        let legacy = render_semantic_pair(
                            TabsRenderMode {
                                semantic: false,
                                close_reveal: reveal,
                            },
                            area,
                            &state,
                            &items,
                            None,
                            level,
                            target,
                        );
                        let semantic = render_semantic_pair(
                            TabsRenderMode {
                                semantic: true,
                                close_reveal: reveal,
                            },
                            area,
                            &state,
                            &items,
                            None,
                            level,
                            target,
                        );
                        assert_eq!(
                            semantic.0, legacy.0,
                            "cells differ at {width} / {level:?} / {reveal:?}"
                        );
                        assert_eq!(
                            semantic.1, legacy.1,
                            "parts differ at {width} / {level:?} / {reveal:?}"
                        );
                        assert_eq!(
                            semantic.2, legacy.2,
                            "cursor differs at {width} / {level:?} / {reveal:?}"
                        );
                        assert_eq!(semantic.3.viewport_len, legacy.3.viewport_len);
                        assert_eq!(semantic.3.content_len, items.len());
                        assert_eq!(semantic.3.clip(), area);
                    }
                }
            }
        }
    }

    #[test]
    fn explicit_window_receipt_preserves_warm_overflow_reservation() {
        let items = ["one", "two", "three", "four", "five"];
        let area = Rect::new(0, 0, 18, 2);
        let state = TabsState::default();
        let tabs = Tabs::new(TABS).closable(true);
        let mut legacy = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy_buf = Buffer::empty(area);
        let mut semantic_buf = Buffer::empty(area);

        legacy
            .draw_scene(area, &mut legacy_buf, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        let first = tabs.layout_default(area, area, &state, &items, None);
        semantic
            .draw_scene(area, &mut semantic_buf, |ui, _| {
                tabs.draw_layout(ui, &first).expect("valid first layout");
            })
            .commit_presented();
        assert_eq!(semantic_buf, legacy_buf);

        legacy_buf = Buffer::empty(area);
        semantic_buf = Buffer::empty(area);
        legacy
            .draw_scene(area, &mut legacy_buf, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        let warm = tabs.layout_default(area, area, &state, &items, first.window_receipt());
        assert!(warm.overflow_reserved);
        semantic
            .draw_scene(area, &mut semantic_buf, |ui, _| {
                tabs.draw_layout(ui, &warm).expect("valid warm layout");
            })
            .commit_presented();
        assert_eq!(semantic_buf, legacy_buf);
        assert_eq!(semantic.registry().regions(), legacy.registry().regions());
    }

    struct SegmentedDisplay;

    impl fmt::Display for SegmentedDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("e")?;
            f.write_str("\u{301}")?;
            f.write_str("界 ")
        }
    }

    struct PiecesDisplay(&'static [&'static str]);

    impl fmt::Display for PiecesDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for piece in self.0 {
                f.write_str(piece)?;
            }
            Ok(())
        }
    }

    struct CountingDisplay<'a>(&'a Cell<usize>);

    impl fmt::Display for CountingDisplay<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.0.set(self.0.get() + 1);
            f.write_str("counted")
        }
    }

    struct CountedOwnedItem {
        key: u64,
        segments: &'static [&'static str],
        formats: Cell<usize>,
        keys: Cell<usize>,
        marks: Cell<usize>,
        mark: TabMarks,
    }

    impl fmt::Display for CountedOwnedItem {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.formats.set(self.formats.get() + 1);
            for segment in self.segments {
                f.write_str(segment)?;
            }
            Ok(())
        }
    }

    fn counted_owned_key(item: &CountedOwnedItem) -> ItemKey {
        item.keys.set(item.keys.get() + 1);
        ItemKey::num(item.key)
    }

    fn counted_owned_marks(item: &CountedOwnedItem) -> TabMarks {
        item.marks.set(item.marks.get() + 1);
        item.mark
    }

    struct MutableOwnedItem {
        key: u64,
        label: Cell<&'static str>,
        mark: Cell<TabMarks>,
    }

    impl fmt::Display for MutableOwnedItem {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.label.get())
        }
    }

    fn mutable_owned_key(item: &MutableOwnedItem) -> ItemKey {
        ItemKey::num(item.key)
    }

    fn mutable_owned_marks(item: &MutableOwnedItem) -> TabMarks {
        item.mark.get()
    }

    // Frozen preimage-shaped Debug views: new private planning fields and the
    // new row close_enabled field are deliberately not represented here.
    mod frozen_layout_debug {
        use super::*;

        struct LegacyTabRowLayout<'a> {
            key: &'a ItemKey,
            index: &'a usize,
            logical_content: &'a Rect,
            visible_content: &'a Rect,
            logical_tab: &'a Rect,
            visible_tab: &'a Rect,
            logical_close: &'a Option<Rect>,
            visible_close: &'a Option<Rect>,
            logical_mark: &'a Option<Rect>,
            visible_mark: &'a Option<Rect>,
            mark: &'a Option<TabItemMark>,
            label_segments: &'a Vec<String>,
            label_extent: &'a u16,
        }

        impl fmt::Debug for LegacyTabRowLayout<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("LegacyTabRowLayout")
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

        struct LegacyTabLayout<'a> {
            id: &'a Id,
            area: &'a Rect,
            clip: &'a Rect,
            row_area: &'a Rect,
            used_area: &'a Rect,
            rule_area: &'a Option<Rect>,
            rows: Vec<LegacyTabRowLayout<'a>>,
            first_index: &'a usize,
            active: &'a Option<ItemKey>,
            cursor: &'a Option<ItemKey>,
            first: &'a Option<ItemKey>,
            first_tab_x: &'a u16,
            allow_new: &'a bool,
            new_key_bindings: &'a Option<&'static [Binding<TabsCmd>]>,
            closable: &'a bool,
            closable_keys: &'a Option<Vec<ItemKey>>,
            close_key_bindings: &'a Option<&'static [Binding<TabsCmd>]>,
            close_reveal: &'a TabCloseReveal,
            close_target_w: &'a Option<u16>,
            close_danger_hover: &'a bool,
            close_reveal_press: &'a Option<ItemKey>,
            status: &'a Status,
            rule: &'a Option<TabRule>,
            rule_focus_hidden: &'a Option<bool>,
            passive: &'a bool,
            passive_focused: &'a bool,
            left_overflow: &'a Option<Rect>,
            right_overflow: &'a Option<Rect>,
            new_area: &'a Option<Rect>,
            status_area: &'a Option<Rect>,
            item_marks_enabled: &'a bool,
            right_limit: &'a u16,
            hidden_right: &'a usize,
            logical_area: &'a Rect,
            visible_area: &'a Rect,
            viewport_len: &'a usize,
            content_len: &'a usize,
            overflow_reserved: &'a bool,
        }

        impl fmt::Debug for LegacyTabLayout<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct("LegacyTabLayout")
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

        fn row(value: &super::TabRowLayout) -> LegacyTabRowLayout<'_> {
            LegacyTabRowLayout {
                key: &value.key,
                index: &value.index,
                logical_content: &value.logical_content,
                visible_content: &value.visible_content,
                logical_tab: &value.logical_tab,
                visible_tab: &value.visible_tab,
                logical_close: &value.logical_close,
                visible_close: &value.visible_close,
                logical_mark: &value.logical_mark,
                visible_mark: &value.visible_mark,
                mark: &value.mark,
                label_segments: &value.label_segments,
                label_extent: &value.label_extent,
            }
        }

        pub(super) fn row_string(value: &super::TabRowLayout) -> String {
            format!("{:?}", row(value)).replace("LegacyTabRowLayout", "TabRowLayout")
        }

        pub(super) fn layout_string(value: &super::TabLayout) -> String {
            let rows = value.rows.iter().map(row).collect();
            format!(
                "{:?}",
                LegacyTabLayout {
                    id: &value.id,
                    area: &value.area,
                    clip: &value.clip,
                    row_area: &value.row_area,
                    used_area: &value.used_area,
                    rule_area: &value.rule_area,
                    rows,
                    first_index: &value.first_index,
                    active: &value.active,
                    cursor: &value.cursor,
                    first: &value.first,
                    first_tab_x: &value.first_tab_x,
                    allow_new: &value.allow_new,
                    new_key_bindings: &value.new_key_bindings,
                    closable: &value.closable,
                    closable_keys: &value.closable_keys,
                    close_key_bindings: &value.close_key_bindings,
                    close_reveal: &value.close_reveal,
                    close_target_w: &value.close_target_w,
                    close_danger_hover: &value.close_danger_hover,
                    close_reveal_press: &value.close_reveal_press,
                    status: &value.status,
                    rule: &value.rule,
                    rule_focus_hidden: &value.rule_focus_hidden,
                    passive: &value.passive,
                    passive_focused: &value.passive_focused,
                    left_overflow: &value.left_overflow,
                    right_overflow: &value.right_overflow,
                    new_area: &value.new_area,
                    status_area: &value.status_area,
                    item_marks_enabled: &value.item_marks_enabled,
                    right_limit: &value.right_limit,
                    hidden_right: &value.hidden_right,
                    logical_area: &value.logical_area,
                    visible_area: &value.visible_area,
                    viewport_len: &value.viewport_len,
                    content_len: &value.content_len,
                    overflow_reserved: &value.overflow_reserved,
                }
            )
            .replace("LegacyTabLayout", "TabLayout")
            .replace("LegacyTabRowLayout", "TabRowLayout")
        }
    }

    struct KeyedDisplay {
        key: u64,
        label: &'static str,
    }

    impl fmt::Display for KeyedDisplay {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.label)
        }
    }

    #[test]
    fn semantic_layout_preserves_display_write_boundaries_and_custom_rows_stay_legacy() {
        let area = Rect::new(0, 0, 20, 2);
        let items = [SegmentedDisplay];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS);
        let layout = tabs.layout_default(area, area, &state, &items, None);
        let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy = Buffer::empty(area);
        let mut semantic = Buffer::empty(area);
        legacy_runtime
            .draw_scene(area, &mut legacy, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        semantic_runtime
            .draw_scene(area, &mut semantic, |ui, _| {
                tabs.draw_layout(ui, &layout)
                    .expect("valid segmented layout");
            })
            .commit_presented();
        assert_eq!(semantic, legacy);

        let calls = Cell::new(0usize);
        let custom = Tabs::new(TABS).row(|_: &&str, row: &mut RowUi<'_>| {
            calls.set(calls.get() + 1);
            row.label("custom");
        });
        let mut custom_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut custom_buffer = Buffer::empty(area);
        custom_runtime
            .draw_scene(area, &mut custom_buffer, |ui, area| {
                custom.draw(ui, area, &state, &["item"]);
            })
            .commit_presented();
        assert_eq!(calls.get(), 1);
        assert_eq!(
            custom_buffer
                .cell(Position::new(1, 0))
                .map(BufferCell::symbol),
            Some("c")
        );
    }

    #[test]
    fn semantic_layout_matches_legacy_for_left_clipped_segmented_labels() {
        let area = Rect::new(0, 0, 30, 3);
        let clip = Rect::new(5, 0, 20, 3);
        let items = [PiecesDisplay(&["ab", "cd"]), PiecesDisplay(&["next"])];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS).closable(true);
        let layout = tabs.layout_default(area, clip, &state, &items, None);
        let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy = Buffer::empty(area);
        let mut semantic = Buffer::empty(area);

        legacy_runtime
            .draw_scene(area, &mut legacy, |ui, _| {
                ui.with_area(clip, |ui| tabs.draw(ui, area, &state, &items));
            })
            .commit_presented();
        semantic_runtime
            .draw_scene(area, &mut semantic, |ui, _| {
                ui.with_area(clip, |ui| {
                    tabs.draw_layout(ui, &layout)
                        .expect("same tabs configuration draws its planned layout");
                });
            })
            .commit_presented();

        assert_eq!(semantic, legacy);
        assert_eq!(
            semantic_runtime.registry().regions(),
            legacy_runtime.registry().regions()
        );
        assert_eq!(semantic_runtime.cursor(), legacy_runtime.cursor());
    }

    #[test]
    fn planned_draw_consumes_owned_text_and_rejects_a_different_configuration() {
        let area = Rect::new(0, 0, 20, 2);
        let formats = Cell::new(0);
        let items = [CountingDisplay(&formats)];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS).closable(true);
        let layout = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(formats.get(), 1);
        assert_eq!(
            layout
                .part_bounds(PartRef::item(Part::TAB, ItemKey::index(0)))
                .map(|(logical, visible)| (logical.width, visible.width)),
            Some((11, 11))
        );
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                tabs.draw_layout(ui, &layout)
                    .expect("same configuration draws the plan");
            })
            .commit_presented();
        assert_eq!(formats.get(), 1, "draw must use owned formatted text");
        let mismatch = Cell::new(None);
        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                mismatch.set(Some(
                    Tabs::<CountingDisplay<'_>>::new(TABS).draw_layout(ui, &layout),
                ));
            })
            .commit_presented();
        assert_eq!(mismatch.get(), Some(Err(TabLayoutMismatch)));
    }

    #[test]
    fn planned_public_eq_and_debug_keep_preimage_fields_only() {
        let items = vec![
            MutableOwnedItem {
                key: 1,
                label: Cell::new("a"),
                mark: Cell::new(TabMarks::default()),
            },
            MutableOwnedItem {
                key: 2,
                label: Cell::new("b"),
                mark: Cell::new(TabMarks::default()),
            },
            MutableOwnedItem {
                key: 3,
                label: Cell::new("c"),
                mark: Cell::new(TabMarks::default()),
            },
            MutableOwnedItem {
                key: 4,
                label: Cell::new("hidden"),
                mark: Cell::new(TabMarks::default()),
            },
        ];
        let tabs = Tabs::new(TABS)
            .key(mutable_owned_key as fn(&MutableOwnedItem) -> ItemKey)
            .item_marks(mutable_owned_marks as fn(&MutableOwnedItem) -> TabMarks);
        let state = TabsState::default();
        let area = Rect::new(0, 0, 12, 2);
        let original = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(original.rows().len(), 3, "the fourth item is unallocated");
        assert_eq!(
            format!("{original:?}"),
            frozen_layout_debug::layout_string(&original),
            "TabLayout Debug must match the frozen preimage field rendering"
        );
        assert_eq!(
            format!("{:?}", original.rows()[0]),
            frozen_layout_debug::row_string(&original.rows()[0]),
            "TabRowLayout Debug must match the frozen preimage field rendering"
        );

        let old_layout_debug = format!("{original:?}");
        let old_rows = original.rows().to_vec();
        let mut only_new_row_flag_changed = original.rows()[0].clone();
        only_new_row_flag_changed.close_enabled = !only_new_row_flag_changed.close_enabled;
        assert_eq!(only_new_row_flag_changed, original.rows()[0]);
        assert_eq!(
            format!("{only_new_row_flag_changed:?}"),
            frozen_layout_debug::row_string(&only_new_row_flag_changed)
        );

        items[3].label.set("a longer unseen label");
        items[3].mark.set(TabMarks::default().error(true));
        let hidden_changed = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(original, hidden_changed);
        assert_eq!(old_rows.as_slice(), hidden_changed.rows());
        assert_eq!(format!("{hidden_changed:?}"), old_layout_debug);
        assert_ne!(original.owned_items, hidden_changed.owned_items);

        items[0].label.set("visible label changed");
        let visible_changed = tabs.layout_default(area, area, &state, &items, None);
        assert_ne!(original, visible_changed);
        assert_ne!(format!("{visible_changed:?}"), old_layout_debug);
    }

    #[test]
    fn planned_empty_area_preserves_preimage_callback_and_key_lookup_counts() {
        let cases = [
            (Rect::new(0, 0, 0, 2), None, 1, [0, 0, 0], 0),
            (
                Rect::new(0, 0, 5, 0),
                Some(ItemKey::num(2)),
                1,
                [0, 1, 0],
                1,
            ),
            (
                Rect::new(0, 0, 0, 1),
                Some(ItemKey::num(3)),
                1,
                [1, 2, 1],
                2,
            ),
        ];

        for (area, wanted_first, hint, expected_keys, expected_first_index) in cases {
            let items = [
                CountedOwnedItem {
                    key: 1,
                    segments: &["one"],
                    formats: Cell::new(0),
                    keys: Cell::new(0),
                    marks: Cell::new(0),
                    mark: TabMarks::default().dirty(true),
                },
                CountedOwnedItem {
                    key: 2,
                    segments: &["two"],
                    formats: Cell::new(0),
                    keys: Cell::new(0),
                    marks: Cell::new(0),
                    mark: TabMarks::default().done(true),
                },
                CountedOwnedItem {
                    key: 3,
                    segments: &["three"],
                    formats: Cell::new(0),
                    keys: Cell::new(0),
                    marks: Cell::new(0),
                    mark: TabMarks::default().error(true),
                },
            ];
            let tabs = Tabs::new(TABS)
                .key(counted_owned_key as fn(&CountedOwnedItem) -> ItemKey)
                .item_marks(counted_owned_marks as fn(&CountedOwnedItem) -> TabMarks);
            let state = TabsState {
                first: wanted_first,
                first_index: hint,
                ..TabsState::default()
            };
            let layout = tabs.layout_default(area, area, &state, &items, None);

            assert_eq!(layout.first_index(), expected_first_index);
            assert!(layout.owned_items.is_none());
            assert!(layout.terminal_first_index.is_none());
            assert!(layout.owned_snapshot_is_consistent());
            for (item, expected_key_calls) in items.iter().zip(expected_keys) {
                assert_eq!(
                    item.formats.get(),
                    0,
                    "empty geometry does not format items"
                );
                assert_eq!(
                    item.marks.get(),
                    0,
                    "empty geometry does not map item marks"
                );
                assert_eq!(
                    item.keys.get(),
                    expected_key_calls,
                    "matches old index_of lookup"
                );
            }

            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            let result = Cell::new(None);
            runtime
                .draw_scene(area, &mut buffer, |ui, _| {
                    result.set(Some(tabs.draw_layout(ui, &layout)));
                })
                .commit_presented();
            assert_eq!(result.get(), Some(Ok(area)));
            assert!(runtime.diagnostics().is_empty());
            for (item, expected_key_calls) in items.iter().zip(expected_keys) {
                assert_eq!(item.formats.get(), 0);
                assert_eq!(item.marks.get(), 0);
                assert_eq!(
                    item.keys.get(),
                    expected_key_calls,
                    "planned empty draw is inert"
                );
            }
        }

        let items = [
            CountedOwnedItem {
                key: 11,
                segments: &["left"],
                formats: Cell::new(0),
                keys: Cell::new(0),
                marks: Cell::new(0),
                mark: TabMarks::default().dirty(true),
            },
            CountedOwnedItem {
                key: 12,
                segments: &["right"],
                formats: Cell::new(0),
                keys: Cell::new(0),
                marks: Cell::new(0),
                mark: TabMarks::default().done(true),
            },
        ];
        let tabs = Tabs::new(TABS)
            .key(counted_owned_key as fn(&CountedOwnedItem) -> ItemKey)
            .item_marks(counted_owned_marks as fn(&CountedOwnedItem) -> TabMarks);
        let area = Rect::new(0, 0, 12, 2);
        let clip = Rect::new(40, 0, 0, 2);
        assert!(!area.is_empty());
        assert!(clip.is_empty());
        let layout = tabs.layout_default(area, clip, &TabsState::default(), &items, None);
        assert_eq!(
            layout.owned_items.as_ref().map_or(0, |items| items.len()),
            2
        );
        for item in &items {
            assert_eq!(
                item.formats.get(),
                1,
                "nonempty but fully clipped area still plans"
            );
            assert_eq!(item.keys.get(), 1);
            assert_eq!(item.marks.get(), 1);
        }
    }

    #[test]
    fn planned_owned_measurement_visits_zero_one_and_many_items_once_per_fresh_plan() {
        const SEGMENTED: &[&str] = &["e", "\u{301}", "界", " "];
        const SHORT: &[&str] = &["x"];
        const WIDE: &[&str] = &["wide", " ", "label "];
        const CONTROL: &[&str] = &["\t", "tail "];

        for count in [0usize, 1, 8] {
            let items: Vec<_> = (0..count)
                .map(|index| CountedOwnedItem {
                    key: index as u64 + 1,
                    segments: match index % 4 {
                        0 => SEGMENTED,
                        1 => SHORT,
                        2 => WIDE,
                        _ => CONTROL,
                    },
                    formats: Cell::new(0),
                    keys: Cell::new(0),
                    marks: Cell::new(0),
                    mark: if index % 3 == 0 {
                        TabMarks::default().dirty(true)
                    } else {
                        TabMarks::default()
                    },
                })
                .collect();
            let close_keys: Vec<_> = items
                .iter()
                .filter(|item| item.key % 2 == 0)
                .map(|item| ItemKey::num(item.key))
                .collect();
            let tabs = Tabs::new(TABS)
                .key(counted_owned_key as fn(&CountedOwnedItem) -> ItemKey)
                .item_marks(counted_owned_marks as fn(&CountedOwnedItem) -> TabMarks)
                .closable_keys(&close_keys)
                .allow_new(true)
                .status(Status::Busy);
            let state = TabsState::default();
            let area = Rect::new(0, 0, 16, 2);
            let layout = tabs.layout_default(area, area, &state, &items, None);

            assert_eq!(
                layout.owned_items.as_ref().map_or(0, |items| items.len()),
                count
            );
            assert_eq!(layout.content_len, count);
            if count == 8 {
                assert!(layout.viewport_len < count, "fixture includes unseen items");
            }
            for item in &items {
                assert_eq!(item.formats.get(), 1, "one Display pass per item");
                assert_eq!(item.keys.get(), 1, "one key resolution per item");
                assert_eq!(item.marks.get(), 1, "one mark resolution per item");
            }

            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, _| {
                    tabs.draw_layout(ui, &layout)
                        .expect("the fresh plan matches its Tabs configuration");
                })
                .commit_presented();
            for item in &items {
                assert_eq!(item.formats.get(), 1, "planned draw does not format");
                assert_eq!(item.keys.get(), 1, "planned draw uses captured keys");
                assert_eq!(item.marks.get(), 1, "planned draw uses captured marks");
            }
            if let Some(owned_items) = layout.owned_items.as_ref() {
                assert_eq!(owned_items.len(), count);
                if !owned_items.is_empty() {
                    let mut inconsistent = layout.clone();
                    inconsistent
                        .owned_items
                        .as_mut()
                        .expect("nonempty plan stores its owned snapshot")[0]
                        .key = ItemKey::num(u64::MAX);
                    let rejected = Cell::new(None);
                    runtime
                        .draw_scene(area, &mut buffer, |ui, _| {
                            rejected.set(Some(tabs.draw_layout(ui, &inconsistent)));
                        })
                        .commit_presented();
                    assert_eq!(rejected.get(), Some(Err(TabLayoutMismatch)));
                }
            }

            let warm = tabs.layout_default(area, area, &state, &items, layout.window_receipt());
            assert_eq!(
                warm.owned_items.as_ref().map_or(0, |items| items.len()),
                count
            );
            assert_eq!(warm.overflow_reserved, layout.viewport_len < count);
            for item in &items {
                assert_eq!(item.formats.get(), 2, "a fresh plan remeasures all items");
                assert_eq!(item.keys.get(), 2, "a fresh plan resolves all keys");
                assert_eq!(item.marks.get(), 2, "a fresh plan resolves all marks");
            }
        }
    }

    #[test]
    fn planned_terminal_start_reuses_current_geometry_and_clipping_policy() {
        let area = Rect::new(0, 0, 24, 2);
        let items = vec!["a"; 8];
        let state = TabsState::default();
        let tabs = Tabs::new(TABS);
        let cold = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(cold.viewport_len, 6);
        assert_eq!(cold.terminal_first_index, Some(4));

        let warm = tabs.layout_default(
            area,
            area,
            &state,
            &items,
            Some(TabWindowReceipt {
                viewport_len: 3,
                content_len: items.len(),
            }),
        );
        assert!(warm.overflow_reserved);
        assert_eq!(warm.terminal_first_index, Some(4));

        let clipped_items = ["a", "a", "a", "a", "a", "a", "a", "abcdefghijklmnop"];
        let clip = Rect::new(18, 0, 6, 2);
        let scrolled = TabsState {
            first: Some(ItemKey::index(4)),
            first_index: 4,
            ..TabsState::default()
        };
        let clipped = tabs.layout_default(area, clip, &scrolled, &clipped_items, None);
        assert_eq!(
            clipped.terminal_first_index,
            Some(4),
            "the existing allocator includes the partially visible clipped final tab"
        );
        let final_row = clipped.rows.last().expect("suffix includes the final item");
        assert_eq!(final_row.index, 7);
        assert_eq!(final_row.label_extent, 2);
        assert_eq!(final_row.logical_tab.width, 4);
        assert_eq!(final_row.visible_tab.width, 2);

        // The exact terminal predicate is not monotone in candidate start:
        // this clip shows the final item at start 4 but hides it at start 5.
        // Keep the retained candidate domain in ascending order; do not use a
        // monotone-search assumption inside it.
        let owned = clipped
            .owned_items
            .as_ref()
            .expect("nonempty clipped plans retain owned rows");
        let mut scratch = Vec::new();
        let start_four = tabs.walk_default_owned(area, clip, owned, 4, None, &mut scratch, |_| {});
        let start_five = tabs.walk_default_owned(area, clip, owned, 5, None, &mut scratch, |_| {});
        let at_four = start_four
            .last_row
            .expect("suffix at four allocates final item");
        let at_five = start_five
            .last_row
            .expect("suffix at five allocates final item");
        assert_eq!(at_four.index, 7);
        assert_eq!(at_four.logical_tab, Rect::new(16, 0, 4, 1));
        assert_eq!(at_four.visible_tab, Rect::new(18, 0, 2, 1));
        assert_eq!(at_five.index, 7);
        assert_eq!(at_five.logical_tab, Rect::new(12, 0, 4, 1));
        assert!(at_five.visible_tab.is_empty());

        let too_narrow = Rect::new(0, 0, 10, 2);
        let unrepresentable = tabs.layout_default(too_narrow, too_narrow, &state, &items, None);
        assert_eq!(unrepresentable.terminal_first_index, None);
    }

    #[test]
    fn semantic_rows_and_parts_keep_stable_keys_after_reordering() {
        let items = [
            KeyedDisplay {
                key: 30,
                label: "third",
            },
            KeyedDisplay {
                key: 10,
                label: "first",
            },
            KeyedDisplay {
                key: 20,
                label: "second",
            },
        ];
        let mut state = TabsState::default();
        state.set_active(1, ItemKey::num(20));
        state.first = Some(ItemKey::num(10));
        state.first_index = 0;
        let tabs = Tabs::new(TABS).key(|item: &KeyedDisplay| ItemKey::num(item.key));
        let area = Rect::new(0, 0, 24, 2);
        let layout = tabs.layout_default(area, area, &state, &items, None);
        assert_eq!(layout.first_index(), 1);
        assert_eq!(
            layout.rows().iter().map(|row| row.key).collect::<Vec<_>>(),
            [ItemKey::num(10), ItemKey::num(20)]
        );
        assert_eq!(layout.active_key(), Some(ItemKey::num(20)));
        assert!(
            layout
                .part_bounds(PartRef::item(Part::TAB, ItemKey::num(20)))
                .is_some()
        );

        let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut legacy = Buffer::empty(area);
        let mut semantic = Buffer::empty(area);
        legacy_runtime
            .draw_scene(area, &mut legacy, |ui, area| {
                tabs.draw(ui, area, &state, &items);
            })
            .commit_presented();
        semantic_runtime
            .draw_scene(area, &mut semantic, |ui, _| {
                tabs.draw_layout(ui, &layout)
                    .expect("same keyed tabs configuration draws its plan");
            })
            .commit_presented();
        assert_eq!(semantic, legacy);
        assert_eq!(
            semantic_runtime.registry().regions(),
            legacy_runtime.registry().regions()
        );
    }

    #[test]
    fn mixed_close_policy_layout_matches_legacy_across_clipping_and_overflow_frames() {
        let items = [
            PiecesDisplay(&["pin"]),
            PiecesDisplay(&["e", "\u{301}", "界"]),
            PiecesDisplay(&["three"]),
            PiecesDisplay(&["四"]),
            PiecesDisplay(&["last"]),
        ];
        let eligible = [ItemKey::index(1), ItemKey::index(3)];
        for reveal in [TabCloseReveal::Always, TabCloseReveal::Hover] {
            let tabs = Tabs::new(TABS)
                .allow_new(true)
                .closable(false)
                .closable_keys(&eligible)
                .close_reveal(reveal);
            let mut offset = TabsState::default();
            offset.set_active(4, ItemKey::index(4));
            offset.first = Some(ItemKey::index(3));
            offset.first_index = 3;
            let states = [TabsState::default(), offset];
            let cases = [
                (Rect::new(0, 0, 0, 0), Rect::new(0, 0, 0, 0)),
                (Rect::new(0, 0, 1, 1), Rect::new(0, 0, 1, 1)),
                (Rect::new(0, 0, 2, 2), Rect::new(0, 0, 2, 2)),
                (Rect::new(0, 0, 9, 1), Rect::new(0, 0, 9, 1)),
                (Rect::new(0, 0, 24, 2), Rect::new(0, 0, 24, 2)),
                (Rect::new(0, 0, 32, 2), Rect::new(5, 0, 20, 2)),
            ];

            for state in &states {
                for (area, clip) in cases {
                    let mut legacy_runtime = Runtime::new(Stub::default(), Theme::junie());
                    let mut semantic_runtime = Runtime::new(Stub::default(), Theme::junie());
                    let mut legacy = Buffer::empty(area);
                    let mut semantic = Buffer::empty(area);
                    let mut previous = None;

                    for _ in 0..2 {
                        let layout = tabs.layout_default(area, clip, state, &items, previous);
                        assert_eq!(
                            layout.part_bounds(PartRef::item(Part::CLOSE, ItemKey::index(0))),
                            None
                        );
                        assert_eq!(
                            layout
                                .part_bounds(PartRef::item(Part::CLOSE, ItemKey::index(1)))
                                .is_some(),
                            layout.rows().iter().any(|row| row.key == ItemKey::index(1))
                        );

                        legacy_runtime
                            .draw_scene(area, &mut legacy, |ui, _| {
                                ui.with_area(clip, |ui| tabs.draw(ui, area, state, &items));
                            })
                            .commit_presented();
                        semantic_runtime
                            .draw_scene(area, &mut semantic, |ui, _| {
                                ui.with_area(clip, |ui| {
                                    tabs.draw_layout(ui, &layout)
                                        .expect("same close policy draws its planned layout");
                                });
                            })
                            .commit_presented();

                        assert_eq!(semantic, legacy, "cells differ for {area:?} / {clip:?}");
                        assert_eq!(
                            semantic_runtime.registry().regions(),
                            legacy_runtime.registry().regions(),
                            "parts differ for {area:?} / {clip:?}"
                        );
                        assert_eq!(
                            semantic_runtime.cursor(),
                            legacy_runtime.cursor(),
                            "cursor differs for {area:?} / {clip:?}"
                        );
                        previous = layout.window_receipt();
                    }
                }
            }
        }
    }

    #[test]
    fn planned_draw_rejects_changed_close_policy_before_paint_or_registration() {
        let area = Rect::new(0, 0, 24, 2);
        let items = ["pinned", "open"];
        let state = TabsState::default();
        let planned_keys = [ItemKey::index(0)];
        let renderer_keys = [ItemKey::index(1)];
        let planned = Tabs::new(TABS).closable_keys(&planned_keys);
        let layout = planned.layout_default(area, area, &state, &items, None);
        let renderer = Tabs::<&str>::new(TABS).closable_keys(&renderer_keys);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        let unpainted = buffer.clone();
        let result = Cell::new(None);

        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                result.set(Some(renderer.draw_layout(ui, &layout)));
            })
            .commit_presented();

        assert_eq!(result.get(), Some(Err(TabLayoutMismatch)));
        assert_eq!(buffer, unpainted);
        assert_eq!(
            runtime
                .registry()
                .area_of_part(TABS, PartRef::item(Part::CLOSE, ItemKey::index(0))),
            None
        );
    }

    #[test]
    fn planned_close_bounds_are_potential_geometry_and_reveal_mismatch_is_preflighted() {
        let area = Rect::new(0, 0, 36, 2);
        let items = ["active", "idle"];
        let mut state = TabsState::default();
        state.set_active(0, ItemKey::index(0));
        let eligible = [ItemKey::index(0), ItemKey::index(1)];
        let planned = Tabs::new(TABS)
            .closable_keys(&eligible)
            .close_reveal(TabCloseReveal::Hover);
        let layout = planned.layout_default(area, area, &state, &items, None);
        let idle_close = PartRef::item(Part::CLOSE, ItemKey::index(1));
        assert!(layout.part_bounds(idle_close).is_some());

        let renderer = Tabs::<&str>::new(TABS)
            .closable_keys(&eligible)
            .close_reveal(TabCloseReveal::Always);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        let unpainted = buffer.clone();
        let result = Cell::new(None);

        runtime
            .draw_scene(area, &mut buffer, |ui, _| {
                result.set(Some(renderer.draw_layout(ui, &layout)));
            })
            .commit_presented();

        assert_eq!(result.get(), Some(Err(TabLayoutMismatch)));
        assert_eq!(buffer, unpainted);
        assert_eq!(runtime.registry().regions(), &[]);
    }

    // Frozen pre-v3 row loop. Keeping this allocating reference under cfg(test)
    // lets the new shared walker prove exact geometry without using itself as
    // the expected-value oracle.
    fn preimage_default_rows<T, K>(
        tabs: &Tabs<'_, T, K, DefaultRow>,
        area: Rect,
        clip: Rect,
        items: &[OwnedTabMeasurement],
        first_index: usize,
        previous: Option<TabWindowReceipt>,
    ) -> Vec<TabRowLayout>
    where
        T: fmt::Display,
        K: KeyFn<T>,
    {
        let len = items.len();
        let first_index = first_index.min(len);
        let overflow_reserved =
            previous.is_some_and(|receipt| receipt.viewport_len < receipt.content_len);
        if area.is_empty() {
            return Vec::new();
        }
        let status_w: u16 = if matches!(tabs.status, Status::Ready) {
            0
        } else {
            2
        };
        let new_w: u16 = if tabs.allow_new { 4 } else { 0 };
        let left_w: u16 = if first_index > 0 { 4 } else { 0 };
        let right_w: u16 = if overflow_reserved || first_index > 0 {
            4
        } else {
            0
        };
        let row0 = first_row(area);
        let right_limit = row0
            .right()
            .saturating_sub(status_w)
            .saturating_sub(right_w)
            .saturating_sub(new_w);
        let mut x = row0.x.saturating_add(left_w);
        let mut rows = Vec::new();
        for (index, item) in items.iter().enumerate().skip(first_index) {
            let mark = item.mark;
            let mark_width = if mark.is_some() { 2 } else { 0 };
            let close_enabled = item.close_enabled;
            let close_w: u16 = if close_enabled { 2 } else { 0 };
            let avail = right_limit.saturating_sub(x);
            if avail < 3 {
                break;
            }
            if mark.is_some() && avail < 1 + 1 + mark_width + 1 + close_w {
                break;
            }
            let content = Rect {
                x: x.saturating_add(1),
                y: row0.y,
                width: avail
                    .saturating_sub(1)
                    .saturating_sub(mark_width)
                    .saturating_sub(u16::from(mark.is_some()))
                    .saturating_sub(if mark.is_some() { close_w } else { 0 }),
                height: 1,
            };
            let label_extent = semantic_label_extent(&item.label_segments, content, clip).max(1);
            let tab_w = 1u16
                .saturating_add(label_extent)
                .saturating_add(mark_width)
                .saturating_add(1)
                .saturating_add(close_w);
            let tab = Rect {
                x,
                y: row0.y,
                width: tab_w.min(avail),
                height: 1,
            };
            let close = close_enabled
                .then(|| close_target_rect(tab, tabs.close_target_w))
                .flatten();
            let logical_mark = mark.map(|_| Rect {
                x: x.saturating_add(1).saturating_add(label_extent),
                y: row0.y,
                width: 2,
                height: 1,
            });
            rows.push(TabRowLayout {
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
                label_segments: item.label_segments.clone(),
                label_extent,
            });
            x = tab.right().saturating_add(1);
        }
        rows
    }

    #[test]
    fn planned_shared_row_walker_matches_preimage_geometry_and_reuses_extent_scratch() {
        let items = [
            OwnedTabMeasurement {
                key: ItemKey::num(1),
                mark: Some(TabItemMark::Dirty),
                close_enabled: true,
                label_segments: ["e", "\u{301}", "界", " "]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            },
            OwnedTabMeasurement {
                key: ItemKey::num(2),
                mark: None,
                close_enabled: false,
                label_segments: ["wide label", " "].into_iter().map(str::to_owned).collect(),
            },
            OwnedTabMeasurement {
                key: ItemKey::num(3),
                mark: Some(TabItemMark::Error),
                close_enabled: true,
                label_segments: ["\t", "👩‍👩‍👧‍👦", "tail "]
                    .into_iter()
                    .map(str::to_owned)
                    .collect(),
            },
            OwnedTabMeasurement {
                key: ItemKey::num(4),
                mark: None,
                close_enabled: true,
                label_segments: ["", "  "].into_iter().map(str::to_owned).collect(),
            },
            OwnedTabMeasurement {
                key: ItemKey::num(5),
                mark: Some(TabItemMark::Done),
                close_enabled: false,
                label_segments: ["last", "界", " "].into_iter().map(str::to_owned).collect(),
            },
        ];
        let tabs = Tabs::<&str>::new(TABS)
            .allow_new(true)
            .status(Status::Busy)
            .close_target_w(4);
        let area = Rect::new(3, 2, 31, 2);
        let clips = [
            area,
            Rect::new(9, 2, 17, 2),
            Rect::new(27, 2, 7, 2),
            Rect::new(45, 2, 0, 2),
        ];
        let receipts = [
            None,
            Some(TabWindowReceipt {
                viewport_len: 2,
                content_len: items.len(),
            }),
        ];

        for clip in clips {
            for previous in receipts {
                let mut scratch = Vec::new();
                // Candidate zero may have a narrower content area than later
                // starts because its left overflow lane is absent. Warm every
                // suffix first so this check compares the stable capacity only
                // after the widest tested row has been seen.
                for first_index in 0..=items.len() {
                    let _ = tabs.walk_default_owned(
                        area,
                        clip,
                        &items,
                        first_index,
                        previous,
                        &mut scratch,
                        |_| {},
                    );
                }
                let warmed_capacity = scratch.capacity();
                for first_index in 0..=items.len() {
                    let expected =
                        preimage_default_rows(&tabs, area, clip, &items, first_index, previous);
                    let mut emitted = Vec::new();
                    let mut emitted_geometry = Vec::new();
                    let walk = tabs.walk_default_owned(
                        area,
                        clip,
                        &items,
                        first_index,
                        previous,
                        &mut scratch,
                        |geometry| {
                            emitted_geometry.push(geometry);
                            emitted.push(
                                geometry.into_tab_row(items[geometry.index].label_segments.clone()),
                            );
                        },
                    );
                    assert_eq!(
                        emitted, expected,
                        "rows differ for clip={clip:?}, first={first_index}, receipt={previous:?}"
                    );
                    assert_eq!(walk.viewport_len, expected.len());
                    let expected_geometry: Vec<_> = expected
                        .iter()
                        .map(DefaultRowGeometry::from_tab_row)
                        .collect();
                    assert_eq!(emitted_geometry, expected_geometry);
                    let expected_last = expected.last().map(DefaultRowGeometry::from_tab_row);
                    assert_eq!(walk.last_row, expected_last);

                    let scalar = tabs.walk_default_owned(
                        area,
                        clip,
                        &items,
                        first_index,
                        previous,
                        &mut scratch,
                        |_| {},
                    );
                    assert_eq!(scalar.viewport_len, expected.len());
                    assert_eq!(scalar.last_row, expected_last);
                    assert_eq!(
                        scratch.capacity(),
                        warmed_capacity,
                        "all suffix probes reuse capacity after every candidate was warmed"
                    );
                }
            }
        }
    }

    fn terminal_fixture_items(count: usize) -> Vec<OwnedTabMeasurement> {
        const SEGMENTS: &[&[&str]] = &[
            &["short"],
            &["e", "\u{301}", "tail "],
            &["界", "joined"],
            &["\t", "👩‍👩‍👧‍👦", "end"],
            &["   "],
            &["wide label"],
            &["x", "\u{301}", "界"],
            &["abcdefghijklmnop"],
        ];
        (0..count)
            .map(|index| OwnedTabMeasurement {
                key: ItemKey::index(index),
                mark: match index % 4 {
                    0 => Some(TabItemMark::Dirty),
                    1 => Some(TabItemMark::Busy),
                    2 => Some(TabItemMark::Error),
                    _ => None,
                },
                close_enabled: index % 3 == 0,
                label_segments: SEGMENTS[index % SEGMENTS.len()]
                    .iter()
                    .map(|segment| (*segment).to_owned())
                    .collect(),
            })
            .collect()
    }

    fn exhaustive_terminal_start<T, K>(
        tabs: &Tabs<'_, T, K, DefaultRow>,
        area: Rect,
        clip: Rect,
        items: &[OwnedTabMeasurement],
        previous: Option<TabWindowReceipt>,
    ) -> Option<usize>
    where
        T: fmt::Display,
        K: KeyFn<T>,
    {
        let final_index = items.len().checked_sub(1)?;
        let mut scratch = Vec::new();
        (0..items.len()).find(|&candidate_first| {
            tabs.walk_default_owned(
                area,
                clip,
                items,
                candidate_first,
                previous,
                &mut scratch,
                |_| {},
            )
            .last_row
            .is_some_and(|row| row.index == final_index && !row.visible_tab.is_empty())
        })
    }

    #[test]
    fn bounded_terminal_search_matches_the_exhaustive_domain() {
        let counts = [0, 1, 2, 5, 8];

        for count in counts {
            let items = terminal_fixture_items(count);
            let receipts = [
                None,
                Some(TabWindowReceipt {
                    viewport_len: 0,
                    content_len: count,
                }),
            ];
            let areas = [
                Rect::new(0, 0, 2, 2),
                Rect::new(0, 0, 4, 2),
                Rect::new(0, 0, 10, 2),
                Rect::new(3, 2, 31, 2),
                Rect::new(u16::MAX - 2, 0, 8, 2),
            ];

            for (status, allow_new, closable) in [
                (Status::Ready, false, false),
                (Status::Busy, true, false),
                (Status::Error, true, true),
            ] {
                let tabs = Tabs::<&str>::new(TABS)
                    .status(status)
                    .allow_new(allow_new)
                    .closable(closable)
                    .close_target_w(5);
                for area in areas {
                    let clips = [
                        area,
                        Rect::new(
                            area.x.saturating_add(area.width / 2),
                            area.y,
                            area.width / 2,
                            1,
                        ),
                        Rect::new(area.x, area.y.saturating_add(4), 5, 1),
                        Rect::new(area.x, area.y, 0, 1),
                    ];
                    for clip in clips {
                        for previous in receipts {
                            let expected =
                                exhaustive_terminal_start(&tabs, area, clip, &items, previous);
                            let mut scratch = Vec::new();
                            let actual = tabs.find_terminal_first_index(
                                area,
                                clip,
                                &items,
                                previous,
                                &mut scratch,
                            );
                            assert_eq!(
                                actual, expected,
                                concat!(
                                    "count={}, status={:?}, allow_new={}, ",
                                    "closable={}, area={:?}, clip={:?}, receipt={:?}"
                                ),
                                count, status, allow_new, closable, area, clip, previous
                            );

                            if !items.is_empty() {
                                let budget =
                                    tabs.default_row_header(area, 1, previous).available_width();
                                let max_rows = max_rows_for_width_budget(budget);
                                let lower =
                                    positive_terminal_candidate_range(count, max_rows).start;
                                let positive_walk = tabs.walk_default_owned(
                                    area,
                                    clip,
                                    &items,
                                    1,
                                    previous,
                                    &mut scratch,
                                    |_| {},
                                );
                                assert_eq!(
                                    positive_walk
                                        .right_limit
                                        .saturating_sub(positive_walk.first_tab_x),
                                    budget,
                                    "positive-start bound uses the exact walker header"
                                );
                                for candidate_first in 1..lower {
                                    let walk = tabs.walk_default_owned(
                                        area,
                                        clip,
                                        &items,
                                        candidate_first,
                                        previous,
                                        &mut scratch,
                                        |_| {},
                                    );
                                    let required_rows = count - candidate_first;
                                    assert!(required_rows > max_rows);
                                    assert!(walk.viewport_len <= max_rows);
                                    assert!(walk.viewport_len < required_rows);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn terminal_row_budget_and_positive_range_use_wide_exact_arithmetic() {
        assert_eq!(max_rows_for_width_budget(0), 0);
        assert_eq!(max_rows_for_width_budget(1), 0);
        assert_eq!(max_rows_for_width_budget(2), 0);
        assert_eq!(max_rows_for_width_budget(3), 1);
        assert_eq!(max_rows_for_width_budget(6), 1);
        assert_eq!(max_rows_for_width_budget(7), 2);
        assert_eq!(max_rows_for_width_budget(u16::MAX), 16_384);

        assert_eq!(positive_terminal_candidate_range(0, 0), 0..0);
        assert_eq!(positive_terminal_candidate_range(1, 0), 1..1);
        assert_eq!(positive_terminal_candidate_range(8, 0), 8..8);
        assert_eq!(positive_terminal_candidate_range(8, 1), 7..8);
        assert_eq!(positive_terminal_candidate_range(8, 8), 1..8);
        assert_eq!(positive_terminal_candidate_range(8, usize::MAX), 1..8);

        let area = Rect::new(3, 2, 31, 2);
        for (status, allow_new, expected_budget, expected_rows) in [
            (Status::Ready, false, 23, 6),
            (Status::Busy, false, 21, 5),
            (Status::Error, true, 17, 4),
        ] {
            let tabs = Tabs::<&str>::new(TABS).status(status).allow_new(allow_new);
            let header = tabs.default_row_header(area, 1, None);
            assert_eq!(header.first_tab_x, 7);
            assert_eq!(header.available_width(), expected_budget);
            assert_eq!(
                max_rows_for_width_budget(header.available_width()),
                expected_rows
            );
        }
    }

    #[test]
    fn bounded_terminal_search_matches_exhaustive_large_input() {
        let items = terminal_fixture_items(128);
        for (status, allow_new, closable) in
            [(Status::Ready, false, false), (Status::Busy, true, true)]
        {
            let tabs = Tabs::<&str>::new(TABS)
                .status(status)
                .allow_new(allow_new)
                .closable(closable);
            for area in [
                Rect::new(0, 0, 4, 2),
                Rect::new(3, 2, 31, 2),
                Rect::new(0, 0, 80, 2),
            ] {
                let clips = [
                    area,
                    Rect::new(
                        area.x.saturating_add(area.width / 2),
                        area.y,
                        area.width / 2,
                        1,
                    ),
                    Rect::new(area.x, area.y.saturating_add(4), 5, 1),
                ];
                for clip in clips {
                    for previous in [
                        None,
                        Some(TabWindowReceipt {
                            viewport_len: 0,
                            content_len: items.len(),
                        }),
                    ] {
                        let expected =
                            exhaustive_terminal_start(&tabs, area, clip, &items, previous);
                        let mut scratch = Vec::new();
                        let actual = tabs.find_terminal_first_index(
                            area,
                            clip,
                            &items,
                            previous,
                            &mut scratch,
                        );
                        assert_eq!(
                            actual, expected,
                            concat!(
                                "large input: status={:?}, allow_new={}, ",
                                "closable={}, area={:?}, clip={:?}, receipt={:?}"
                            ),
                            status, allow_new, closable, area, clip, previous
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn terminal_probe_work_is_bounded_by_fixed_viewport_width() {
        let tabs = Tabs::<&str>::new(TABS);
        let area = Rect::new(0, 0, 80, 2);
        let clip = Rect::new(100, 0, 1, 1);
        let mut measurements = Vec::new();

        for count in [8, 128, 4096] {
            measurements.clear();
            measurements.extend((0..count).map(|index| OwnedTabMeasurement {
                key: ItemKey::index(index),
                mark: None,
                close_enabled: false,
                label_segments: vec!["x".to_owned()],
            }));

            let positive_budget = tabs.default_row_header(area, 1, None).available_width();
            let max_rows = max_rows_for_width_budget(positive_budget);
            let candidates = positive_terminal_candidate_range(count, max_rows);
            let mut scratch = Vec::new();
            let mut probes = 0usize;
            let mut row_visits = 0usize;

            let start_zero =
                tabs.walk_default_owned(area, clip, &measurements, 0, None, &mut scratch, |_| {});
            probes += 1;
            row_visits += start_zero.viewport_len;
            assert!(
                !start_zero
                    .last_row
                    .is_some_and(|row| row.index + 1 == count && !row.visible_tab.is_empty())
            );

            let mut positive_row_visits = 0usize;
            for candidate_first in candidates.clone() {
                let walk = tabs.walk_default_owned(
                    area,
                    clip,
                    &measurements,
                    candidate_first,
                    None,
                    &mut scratch,
                    |_| {},
                );
                probes += 1;
                row_visits += walk.viewport_len;
                positive_row_visits += walk.viewport_len;
                assert!(
                    !walk
                        .last_row
                        .is_some_and(|row| row.index + 1 == count && !row.visible_tab.is_empty())
                );
            }

            assert_eq!(
                tabs.find_terminal_first_index(area, clip, &measurements, None, &mut scratch,),
                None
            );
            assert_eq!(max_rows, 18);
            assert_eq!(probes, 1 + candidates.len());
            assert!(probes <= 1 + max_rows);
            assert!(positive_row_visits <= max_rows * max_rows);
            assert_eq!(row_visits, start_zero.viewport_len + positive_row_visits);
            if count == 8 {
                assert_eq!(probes, 8);
                assert_eq!(row_visits, 36);
            } else {
                assert_eq!(probes, 19);
                assert_eq!(positive_row_visits, 171);
                assert_eq!(row_visits, 191);
            }
        }
    }
}
