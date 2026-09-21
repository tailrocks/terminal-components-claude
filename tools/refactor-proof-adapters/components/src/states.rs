//! The CP-COMMON state axis and per-family applicability.
//!
//! Every Direct family declares its applicable states explicitly. States that
//! the production widget cannot reach are excluded with an evidence-backed
//! reason (widget API shape plus the state requirement); absence never stands
//! in for non-applicability. Runtime-owned states (focus/hover/pressed) are
//! reached through real `Harness` input, never injected.

use crate::families::Family;

/// Observable component state (CP-COMMON plus editing).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ComponentState {
    /// Resting props with no runtime state.
    Base,
    /// Keyboard focus on the widget.
    Focus,
    /// Pointer hover over the widget.
    Hover,
    /// Pointer held down on the widget.
    Pressed,
    /// Current cursor row/cell/tab/step.
    Current,
    /// Chosen/checked value.
    Selected,
    /// Disabled: absorbs input, paints disabled.
    Disabled,
    /// Read-only: navigable, not mutable.
    ReadOnly,
    /// Active/running policy (follow, spin, open).
    Active,
    /// Inactive/paused policy.
    Inactive,
    /// Busy readiness.
    Busy,
    /// Loading readiness.
    Loading,
    /// Error readiness with message.
    Error,
    /// Empty content.
    Empty,
    /// Text edit mode entered through production input.
    Editing,
}

impl ComponentState {
    /// Full state axis.
    pub const ALL: [Self; 15] = [
        Self::Base,
        Self::Focus,
        Self::Hover,
        Self::Pressed,
        Self::Current,
        Self::Selected,
        Self::Disabled,
        Self::ReadOnly,
        Self::Active,
        Self::Inactive,
        Self::Busy,
        Self::Loading,
        Self::Error,
        Self::Empty,
        Self::Editing,
    ];

    /// Identity token.
    #[must_use]
    pub const fn token(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::Focus => "focus",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::Current => "current",
            Self::Selected => "selected",
            Self::Disabled => "disabled",
            Self::ReadOnly => "readonly",
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Busy => "busy",
            Self::Loading => "loading",
            Self::Error => "error",
            Self::Empty => "empty",
            Self::Editing => "editing",
        }
    }

    /// What a widget must support for this state to be applicable.
    #[must_use]
    pub const fn requirement(self) -> &'static str {
        match self {
            Self::Base => "paints with resting props",
            Self::Focus => "is a Tab focus stop",
            Self::Hover => "publishes a hoverable hit region",
            Self::Pressed => "holds pointer-down before release",
            Self::Current => "keeps a navigable cursor distinct from value",
            Self::Selected => "keeps a chosen/checked value",
            Self::Disabled => "has a disabled prop or state",
            Self::ReadOnly => "has a read-only/editable mode split",
            Self::Active => "has an active/running policy or prop",
            Self::Inactive => "has an inactive/paused policy or prop",
            Self::Busy => "has a busy readiness variant or status",
            Self::Loading => "has a loading readiness variant or status",
            Self::Error => "has an error readiness variant or message prop",
            Self::Empty => "has an observable empty-content rendering",
            Self::Editing => "has an explicit text edit mode",
        }
    }
}

/// Applicable states for a Direct family. Composition and Architecture
/// families have no frame states.
#[must_use]
pub const fn applicable_states(family: Family) -> &'static [ComponentState] {
    use ComponentState as S;
    use Family as F;
    match family {
        F::ScrollStateRegion | F::Brand | F::SplitPane => &[S::Base, S::Hover, S::Pressed],
        F::Button | F::Select => &[S::Base, S::Focus, S::Hover, S::Pressed, S::Disabled],
        F::CheckboxToggle => &[
            S::Base,
            S::Focus,
            S::Hover,
            S::Pressed,
            S::Selected,
            S::Disabled,
        ],
        F::RadioGroup | F::Tree => &[
            S::Base,
            S::Focus,
            S::Hover,
            S::Current,
            S::Selected,
            S::Disabled,
        ],
        F::Chips => &[S::Base, S::Hover, S::Pressed, S::Selected, S::Disabled],
        F::FieldChrome => &[S::Base, S::Focus, S::Disabled, S::Error],
        F::TextInput | F::TextArea => &[
            S::Base,
            S::Focus,
            S::Hover,
            S::Editing,
            S::Selected,
            S::Disabled,
            S::ReadOnly,
            S::Empty,
        ],
        F::SecretValidation | F::Form => &[S::Base, S::Focus, S::Editing, S::Disabled, S::Error],
        F::List => &[
            S::Base,
            S::Focus,
            S::Hover,
            S::Pressed,
            S::Current,
            S::Selected,
            S::Disabled,
            S::Empty,
        ],
        F::FilterList => &[S::Base, S::Focus, S::Editing, S::Current, S::Empty],
        F::NavList | F::Table => &[S::Base, S::Focus, S::Current, S::Selected, S::Disabled],
        F::Steps => &[S::Base, S::Current, S::Disabled],
        F::Tabs => &[S::Base, S::Focus, S::Hover, S::Pressed, S::Selected],
        F::PickerCommandPalette => &[S::Base, S::Focus, S::Editing, S::Current, S::Disabled],
        F::PickerChain => &[S::Base, S::Focus, S::Current],
        F::Completion => &[S::Base, S::Current, S::Selected],
        F::Dialog => &[S::Base, S::Focus],
        F::MenuContextMenubar => &[S::Base, S::Hover, S::Pressed, S::Disabled],
        F::HelpOverlay | F::Panel | F::Spinner | F::Hintbar | F::Keyhint | F::TooSmall => {
            &[S::Base]
        }
        F::Wizard | F::Props => &[S::Base, S::Current],
        F::Grid => &[
            S::Base,
            S::Focus,
            S::Hover,
            S::Current,
            S::Selected,
            S::Disabled,
            S::Editing,
        ],
        F::CodeEditor => &[
            S::Base,
            S::Focus,
            S::Editing,
            S::Selected,
            S::ReadOnly,
            S::Empty,
        ],
        F::DiffView => &[S::Base, S::Focus, S::Selected],
        F::TextViewport => &[S::Base, S::Focus, S::Selected, S::Empty],
        F::ScrollPanel => &[S::Base, S::Active, S::Inactive],
        F::EmptyReadiness => &[S::Base, S::Loading, S::Error, S::Empty],
        F::ProgressBar => &[S::Base, S::Busy],
        F::Meter => &[S::Base, S::Busy, S::Error],
        F::StatusbarSegments => &[S::Base, S::Busy, S::Error, S::Empty],
        _ => &[],
    }
}

/// One-line production API shape backing the applicability table.
#[must_use]
pub const fn family_capability(family: Family) -> &'static str {
    match family {
        Family::ScrollStateRegion => {
            "ScrollRegion with pointer-driven thumb; region itself is not a Tab stop"
        }
        Family::Button => "Button with checked/disabled props and typed command",
        Family::Brand => "click-only Brand lockup with label/meta parts",
        Family::CheckboxToggle => "controlled Checkbox/Toggle values with disabled prop",
        Family::RadioGroup => "borrowed keyed group with separate cursor and chosen value",
        Family::Chips => "keyed ChipBar with controlled checked state and close/add actions",
        Family::FieldChrome => {
            "Field chrome plus one child Id; error text prop; no second focus stop"
        }
        Family::TextInput => {
            "caller value/draft lifecycle with edit modes, secret and placeholder props"
        }
        Family::TextArea => "multiline draft lifecycle with edit modes and secret prop",
        Family::SecretValidation => "sealed secret TextInput plus Form validation errors",
        Family::Select => "dropdown Select with open/closed state and disabled prop",
        Family::Form => "Form managing field states by stable Id with validation errors",
        Family::List => "keyed List with cursor, chosen, checked, and disabled items",
        Family::FilterList => "FilterList with query input plus keyed rows; no disabled prop",
        Family::NavList => "NavList with mode plus keyed rows",
        Family::Tree => "keyed Tree with cursor, expansion, and chosen value",
        Family::Steps => "Steps with per-step StepState and current index",
        Family::Tabs => "Tabs with selected tab; no disabled prop",
        Family::PickerCommandPalette => "Picker/CommandPalette with query plus current row",
        Family::PickerChain => "PickerChain with staged current row; no disabled prop",
        Family::Completion => "Completion with current candidate and accept action",
        Family::Dialog => "modal Dialog with focusable actions over a dimmed host",
        Family::MenuContextMenubar => "MenuBar/Menu with hoverable titles and open state",
        Family::HelpOverlay => "static HelpOverlay sections; no input state",
        Family::Wizard => "Wizard with current step",
        Family::Grid => "keyed Grid with cursor, range selection, and inline editor",
        Family::Table => "Grid in read-only table policy; no inline editor",
        Family::CodeEditor => "CodeEditor with caret, edit mode, and read-only mode",
        Family::DiffView => "DiffView with adaptive projection and text selection",
        Family::TextViewport => {
            "TextViewport with one Focusable stop, pointer selection, and retained position"
        }
        Family::ScrollPanel => "Panel plus TextViewport composition with explicit follow policy",
        Family::Panel => "static container Panel; no input state",
        Family::SplitPane => "SplitPane with pointer-driven seam; seam itself is not a Tab stop",
        Family::Props => "PropsList with current row",
        Family::EmptyReadiness => {
            "Empty driven by EmptyState Empty/Loading/Error variants; no Busy variant exists"
        }
        Family::ProgressBar => "ProgressBar with ratio plus Status readiness; no disabled prop",
        Family::Spinner => {
            "Spinner with caller-owned frame prop; animation ticks belong to the host app"
        }
        Family::Meter => "Meter with tone/visual plus Status readiness",
        Family::StatusbarSegments => "StatusBar with items plus Status readiness",
        Family::Hintbar => "static HintBar hints; no input state",
        Family::Keyhint => "static KeyHint; no input state",
        Family::TooSmall => "static narrow-allocation fallback; no input state",
        _ => "no frame states: observed through compositions or architecture predicates",
    }
}

/// Explicit non-applicability records: every non-applicable state plus its
/// evidence-backed reason. Never empty-by-absence: the reason is always present.
#[must_use]
pub fn excluded_states(family: Family) -> Vec<(ComponentState, String)> {
    let applicable = applicable_states(family);
    ComponentState::ALL
        .into_iter()
        .filter(|state| !applicable.contains(state))
        .map(|state| {
            let reason = format!(
                "{}; state needs {}",
                family_capability(family),
                state.requirement()
            );
            (state, reason)
        })
        .collect()
}
