//! The component families (`COMPONENT_ARCHITECTURE.md` §3–§15, Appendix A
//! Slice 4): props structs with consuming builders, caller-owned `XState`s,
//! `update`/`draw` phase methods and per-component binding tables.
//!
//! Every component here follows §13: `X::new(id, …)`, consuming builders,
//! `update(&self, cx, &mut st[, data]) -> Response<XAction>`,
//! `draw(&self, ui, area, &st[, data]) -> Rect`, `measure`, `PARTS`.

pub(crate) mod brand;
pub(crate) mod button;
pub(crate) mod chip;
pub(crate) mod choice;
pub(crate) mod code;
pub(crate) mod completion;
pub(crate) mod dialog;
pub(crate) mod diff;
pub(crate) mod empty;
pub(crate) mod field;
pub(crate) mod filter_list;
pub(crate) mod form;
pub(crate) mod grid;
pub(crate) mod help;
pub(crate) mod hintbar;
pub(crate) mod input;
pub(crate) mod keyhint;
pub(crate) mod list;
pub(crate) mod menu;
pub(crate) mod meter;
pub(crate) mod nav_list;
pub(crate) mod panel;
pub(crate) mod picker;
pub(crate) mod picker_chain;
pub(crate) mod progress;
pub(crate) mod props;
pub(crate) mod scroll_region;
pub(crate) mod select;
pub(crate) mod split;
pub(crate) mod status;
pub(crate) mod steps;
pub(crate) mod tabs;
pub(crate) mod textarea;
pub(crate) mod too_small;
pub(crate) mod tree;
pub(crate) mod viewport;
pub(crate) mod wizard;

pub use brand::Brand;
pub use button::{Button, ButtonCmd};
pub use chip::{ChipBar, ChipBarAction, ChipBarCmd, ChipBarState, LabelChips};
pub use choice::{
    Checkbox, ChoiceCmd, LabelRadio, RadioGroup, RadioGroupAction, RadioGroupState, Toggle,
};
pub use code::{
    CodeAction, CodeCmd, CodeDiagnostic, CodeEditor, CodeEditorState, CodeSeverity, Highlighter,
    Segmenter, TabBehavior,
};
pub use completion::{
    Completion, CompletionAction, CompletionCmd, CompletionController, CompletionState,
};
pub use dialog::{Dialog, DialogAction, DialogCmd, DialogState};
pub use diff::{DiffLineKind, DiffMode, DiffRow, DiffSource, DiffView, DiffViewState};
pub use empty::Empty;
pub use field::Field;
pub use filter_list::{FilterList, FilterListAction, FilterListCmd, FilterListState};
pub use form::{
    EnterPolicy, FieldKind, FieldMut, FieldRef, FieldSpan, FieldSpec, Form, FormAction, FormData,
    FormState, GroupKey,
};
pub use grid::{
    CellAction, CellRef, Column, ColumnKey, EditIntent, GRID_MAX_COLUMNS, Grid, GridAction,
    GridCmd, GridEditor, GridModel, GridState, NavUnit, SortDir,
};
pub use help::{HelpAction, HelpCmd, HelpOverlay, HelpOverlayState, HelpSection};
pub use hintbar::{DerivedHintBar, HintBar};
pub use input::{BlurPolicy, EditPhase, TextAction, TextCmd, TextInput, TextInputState};
pub use keyhint::KeyHint;
pub use list::{List, ListAction, ListCmd, ListState};
pub use menu::{ContextMenu, Menu, MenuAction, MenuBar, MenuCmd, MenuItem, MenuState};
pub use meter::{Meter, MeterTone, MeterVisual};
pub use nav_list::{BadgeFn, NavList, NavListAction, NavListCmd, NavListState, NavMode};
pub use panel::{Panel, PanelKind};
pub use picker::{
    AsItem, CommandPalette, Item, ItemRow, Picker, PickerAction, PickerState, ScopeKey,
};
pub use picker_chain::{
    PickerChain, PickerChainAction, PickerChainCmd, PickerChainState, PickerStage,
};
pub use progress::{ProgressBar, Spinner};
pub use props::{Props, PropsAction, PropsCmd, PropsList, PropsRow, PropsState, PropsValue};
pub use scroll_region::ScrollRegion;
pub use select::{LabelSelect, Select, SelectAction, SelectCmd, SelectState};
pub use split::{SplitAction, SplitCmd, SplitPane, SplitPaneState};
pub use status::{Emphasis, Group, MAX_ITEMS, StatusAction, StatusBar, StatusItem};
pub use steps::{StepState, Steps, StepsAction, StepsCmd, StepsState};
pub use tabs::{Tabs, TabsAction, TabsCmd, TabsState};
pub use textarea::{TextArea, TextAreaState};
pub use too_small::TooSmall;
pub use tree::{NodeKind, Tree, TreeAction, TreeCmd, TreeNode, TreeState};
pub use viewport::{
    CellPos, TextViewport, ViewportAction, ViewportCmd, ViewportLine, ViewportState,
};
#[cfg(feature = "testing")]
pub use viewport::{ViewportWorkProbe, ViewportWorkSnapshot};
pub use wizard::{Wizard, WizardAction, WizardCmd, WizardState, WizardStep};

use ratatui_core::layout::Rect;
use ratatui_core::style::Style;

use crate::id::Id;
use crate::theme::GlyphRole;
use crate::ui::Ui;

pub(crate) use crate::author::PartStyle;

/// A replaced part: the component keeps layout, hit registration, focus and
/// state; the closure paints the part's rect.
pub(crate) type SlotFn<'a> = &'a dyn Fn(&mut Ui<'_>, Rect);

/// The first row of `area`, or an empty rect.
pub(crate) const fn first_row(area: Rect) -> Rect {
    Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: if area.height == 0 { 0 } else { 1 },
    }
}

/// A one-cell-wide column of `area` at `x`, or an empty rect when `x` is
/// outside `area`.
///
/// The bound is enforced on **both** sides. Callers anchor a cell to the right
/// edge with `area.right().saturating_sub(n)`, which lands left of `area.x`
/// whenever `area.width < n` — at `area.width == 1`, `right() - 2` is
/// `area.x - 1`. Checking only the right edge made that a paintable cell one
/// column outside the component, so the helper enforces the whole extent its
/// name promises rather than leaving each caller to clamp.
pub(crate) const fn cell_at(area: Rect, x: u16) -> Rect {
    Rect {
        x,
        y: area.y,
        width: if x >= area.x && x < area.x.saturating_add(area.width) {
            1
        } else {
            0
        },
        height: area.height,
    }
}

/// Paint the mono pressed bracket into two cells reserved by the component.
pub(crate) fn paint_pressed_bracket(ui: &mut Ui<'_>, left: Rect, right: Rect, style: Style) {
    ui.glyph(left, GlyphRole::PressLeft, style);
    ui.glyph(right, GlyphRole::PressRight, style);
}

/// `area` shifted right by `by` columns, shrinking its width.
pub(crate) const fn shift(area: Rect, by: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(by),
        y: area.y,
        width: area.width.saturating_sub(by),
        height: area.height,
    }
}

/// Folds a component's per-intent outcomes into one `Response<A>`: an
/// action wins over a repaint, a repaint over a bare consume.
pub(crate) struct Acc<A> {
    consumed: bool,
    invalidate: crate::response::Invalidate,
    action: Option<A>,
}

impl<A> Acc<A> {
    pub(crate) const fn new() -> Self {
        Acc {
            consumed: false,
            invalidate: crate::response::Invalidate::None,
            action: None,
        }
    }

    pub(crate) fn consumed(&mut self) {
        self.consumed = true;
    }

    pub(crate) fn changed(&mut self) {
        self.consumed = true;
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    /// Request a repaint **without** consuming: a notification the component
    /// drains and reacts to, but which must not swallow the input that is
    /// still being dispatched.
    pub(crate) fn repaint(&mut self) {
        self.invalidate = self.invalidate.max(crate::response::Invalidate::Paint);
    }

    pub(crate) fn action(&mut self, a: A) {
        self.changed();
        self.action = Some(a);
    }

    pub(crate) fn fold(&mut self, r: &crate::response::Response<()>) {
        self.consumed |= r.is_consumed();
        self.invalidate = self.invalidate.max(r.invalidate());
    }

    pub(crate) fn finish(self, id: Id) -> crate::response::Response<A> {
        use crate::response::{Invalidate, Response};
        let r = match self.action {
            Some(a) => Response::action(a),
            None if self.consumed => Response::consumed(),
            None if self.invalidate == Invalidate::None => return Response::ignored(),
            None => Response::ignored(),
        };
        let r = match self.invalidate {
            Invalidate::None => r,
            Invalidate::Paint => r.repaint(),
            Invalidate::Layout => r.relayout(),
        };
        r.for_id(id)
    }
}

#[cfg(test)]
mod tests {
    use ratatui_core::layout::Rect;

    use super::cell_at;

    const AREA: Rect = Rect {
        x: 4,
        y: 5,
        width: 3,
        height: 2,
    };

    /// `cell_at` names a cell **of `area`**, so it must yield a zero-width
    /// rect for every `x` outside `area`'s horizontal extent — to the left of
    /// `area.x` exactly as much as at or past `area.x + area.width`. Only the
    /// right edge was enforced, so a caller computing a right-anchored column
    /// with `saturating_sub` (`Select`'s trailing indicator,
    /// `cell_at(area, area.right() - 2)` at `area.width == 1`) got a paintable
    /// cell one column outside the component.
    #[test]
    fn cell_at_is_empty_for_every_x_outside_the_area_on_either_side() {
        // left of `area.x`: empty, both one short and far short
        assert_eq!(cell_at(AREA, AREA.x - 1).width, 0);
        assert_eq!(cell_at(AREA, 0).width, 0);
        // the `Select` case verbatim: a 1-wide area's `right() - 2` is `x - 1`
        let narrow = Rect { width: 1, ..AREA };
        assert_eq!(cell_at(narrow, narrow.right().saturating_sub(2)).width, 0);
        // a zero-width area has no cells at all, including at its own `x`
        let empty = Rect { width: 0, ..AREA };
        assert_eq!(cell_at(empty, empty.x).width, 0);

        // unchanged: every in-range `x` is a one-cell column of `area`
        for x in AREA.x..AREA.right() {
            assert_eq!(
                cell_at(AREA, x),
                Rect {
                    x,
                    y: AREA.y,
                    width: 1,
                    height: AREA.height,
                }
            );
        }

        // unchanged: at or past the right edge is empty, and keeps `x`, `y`
        // and `height` as they were
        assert_eq!(
            cell_at(AREA, AREA.right()),
            Rect {
                x: AREA.right(),
                y: AREA.y,
                width: 0,
                height: AREA.height,
            }
        );
        assert_eq!(cell_at(AREA, AREA.right() + 1).width, 0);
    }
}
