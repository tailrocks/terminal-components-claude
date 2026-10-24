//! Render-site adoption gate for the navigation gutter paint sites (PR #38,
//! controls/navigation ink-API adoption, 2026-10-21).
//!
//! The upgraded shared ink helpers (`Theme::focus_bar_symbol`,
//! `Theme::gutter_in` with `GutterCell::Inherit`) were measured against the
//! frozen `LIST`/`TREE`/`STEPS` row gutters across the reachable state ×
//! tier matrix, rendered through the real widgets (`list.rs` row gutter,
//! `tree.rs::paint_default_row`, `steps.rs::paint_row`). The result decides
//! whether a site can adopt the helpers byte-identically; where it cannot,
//! the recipe-driven local path stays.
//!
//! Measured through the real widgets (matrix run 2026-10-21):
//!
//! - `LIST` and `STEPS` match the helper cell in every probed state at every
//!   tier (rest, checked, focused, focused+checked, focused+pressed,
//!   disabled row; non-navigable and navigable steps).
//! - `TREE` matches except the 16-colour focused+selected row: the accent
//!   tint surface makes `legible_mark(focus, bg)` swap the frozen focus ink
//!   for the primary text ink.
//! - Blocked everywhere, beyond the probed states: a focused+disabled row
//!   blanks the bar but still stamps the focus ink on the reserved cell
//!   (`LIST` at every tier, `TREE`/`STEPS` at the mono tier — the downgrade
//!   rules blank the glyph without clearing the style). `GutterCell::Inherit`
//!   stamps nothing, so converting the site would drop that stamp. The rows
//!   can reach that state through cursor placement edge paths, so the pin is
//!   taken at the resolver level (`frozen_ink_pins` pins the glyph rule;
//!   `list_focused_disabled_blanks_still_stamp_the_focus_ink` below pins the
//!   stamp the conversion would lose).
//!
//! A helper upgrade that closes a divergent state turns the `diverges`
//! assertions red and reopens adoption for that state.

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use ratatui_core::style::Color;
use termrock_core::id::{Id, ItemKey, Part};
use termrock_core::response::StateFlags;
use termrock_navigation::{
    List, ListState, StepState, Steps, StepsState, Tree, TreeNode, TreeState,
};
use termrock_runtime::runtime::stub::Stub;
use termrock_runtime::{ReferenceState, ReferenceTarget, Runtime, Ui};
use termrock_theme::{ColorLevel, Family, FocusBarPolicy, GutterCell, Surface, Theme, Variant};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];

const LIST: Id = Id::root("gate.list");
const TREE: Id = Id::root("gate.tree");
const STEPS: Id = Id::root("gate.steps");

/// Terminal cells store `Color::Reset` for "unset"; styles use `None`.
fn norm(c: Color) -> Option<Color> {
    (c != Color::Reset).then_some(c)
}

fn painted(
    theme: Theme,
    reference: Option<(Id, ReferenceState)>,
    draw: impl FnOnce(&mut Ui<'_>, Rect),
) -> Buffer {
    let area = Rect::new(0, 0, 24, 6);
    let mut runtime = Runtime::new(Stub::default(), theme);
    let mut buffer = Buffer::empty(area);
    runtime
        .draw_scene(area, &mut buffer, |ui, area| {
            if let Some((id, state)) = reference {
                ui.reference(Some(ReferenceTarget::new(id, state)), |ui| draw(ui, area));
            } else {
                draw(ui, area);
            }
        })
        .commit_presented();
    buffer
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Cell {
    symbol: String,
    fg: Option<Color>,
    bg: Option<Color>,
}

fn cell_of(buf: &Buffer, x: u16, y: u16) -> Cell {
    let cell = buf.cell(Position::new(x, y)).expect("cell in bounds");
    Cell {
        symbol: cell.symbol().to_owned(),
        fg: norm(cell.fg),
        bg: norm(cell.bg),
    }
}

/// The bar glyph or blank the gutter cell must show.
fn assert_bar(cell: &Cell, bar: bool, ctx: &str) {
    let expected = if bar { "▎" } else { " " };
    assert_eq!(cell.symbol, expected, "{ctx}: gutter symbol");
}

fn helper_fg(
    theme: &Theme,
    policy: FocusBarPolicy,
    flags: StateFlags,
    bg: Color,
    on_accent: bool,
) -> Option<Color> {
    theme
        .gutter_in(policy, flags, GutterCell::Inherit, bg, on_accent)
        .fg
}

fn helper_blank_stamps_nothing(theme: &Theme, policy: FocusBarPolicy, flags: StateFlags) -> bool {
    theme.focus_bar_symbol(policy, flags) == " "
        && theme
            .gutter_in(
                policy,
                flags,
                GutterCell::Inherit,
                theme.color.surfaces[0],
                false,
            )
            .fg
            .is_none()
}

fn resolver(
    theme: &Theme,
    family: Family,
    part: Part,
    flags: StateFlags,
) -> termrock_theme::Resolved {
    theme.resolve(family, Variant::DEFAULT, part, flags, Surface::Canvas)
}

const FOCUSED: ReferenceState = ReferenceState::FOCUSED;
const FOCUSED_PRESSED: ReferenceState = ReferenceState::FOCUSED.union(ReferenceState::PRESSED);

struct Row {
    name: &'static str,
    disabled: bool,
}

impl std::fmt::Display for Row {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name)
    }
}

fn row_key(r: &Row) -> ItemKey {
    ItemKey::text(r.name)
}

fn list_rows() -> Vec<Row> {
    vec![
        Row {
            name: "one",
            disabled: false,
        },
        Row {
            name: "two",
            disabled: false,
        },
        Row {
            name: "down",
            disabled: true,
        },
    ]
}

fn list_state(checked: bool) -> ListState {
    let rows = list_rows();
    let mut st = ListState::default();
    st.set_cursor(0, row_key(&rows[0]));
    if checked {
        st.checked_mut().insert(row_key(&rows[0]));
    }
    st
}

#[test]
fn list_row_gutters_match_the_helper_cell_in_every_probed_state() {
    let base = Theme::junie();
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for (state, checked, reference, cell_flags) in [
            ("rest", false, None, StateFlags::empty()),
            ("checked", true, None, StateFlags::CHECKED),
            ("focused", false, Some(FOCUSED), StateFlags::FOCUSED),
            (
                "focused_checked",
                true,
                Some(FOCUSED),
                StateFlags::FOCUSED.union(StateFlags::CHECKED),
            ),
            (
                "focused_pressed",
                false,
                Some(FOCUSED_PRESSED),
                StateFlags::FOCUSED.union(StateFlags::PRESSED),
            ),
        ] {
            let rows = list_rows();
            let st = list_state(checked);
            let buf = painted(theme.clone(), reference.map(|s| (LIST, s)), |ui, _area| {
                List::new(LIST)
                    .key(row_key)
                    .draw(ui, Rect::new(0, 1, 24, 3), &st, &rows);
            });
            let cell = cell_of(&buf, 0, 1);
            let ctx = format!("{level:?} LIST {state}");
            assert_bar(&cell, cell_flags.contains(StateFlags::FOCUSED), &ctx);
            if cell_flags.contains(StateFlags::FOCUSED) {
                assert_eq!(
                    cell.fg,
                    helper_fg(
                        &theme,
                        FocusBarPolicy::RowBlanked,
                        cell_flags,
                        cell.bg.expect("row surface"),
                        false,
                    ),
                    "{ctx}: the frozen bar ink equals the helper ink, so the state could adopt"
                );
            }
        }

        // A disabled (never cursor) row: a blank cell in the container ink.
        let rows = list_rows();
        let mut st = list_state(false);
        st.set_cursor(0, row_key(&rows[0]));
        let buf = painted(theme.clone(), None, |ui, _area| {
            List::new(LIST)
                .key(row_key)
                .disabled_item(&|r: &Row| r.disabled)
                .draw(ui, Rect::new(0, 1, 24, 3), &st, &rows);
        });
        let cell = cell_of(&buf, 0, 3);
        let ctx = format!("{level:?} LIST disabled_row");
        assert_bar(&cell, false, &ctx);
        assert_eq!(
            cell.fg,
            resolver(&theme, Family::LIST, Part::CONTAINER, StateFlags::DISABLED)
                .style
                .as_style()
                .fg,
            "{ctx}: the blank keeps the container ink, so the state could adopt"
        );
    }
}

#[test]
fn tree_row_gutters_match_the_helper_cell_except_the_16_colour_selection() {
    let base = Theme::junie();
    let items = ["branch", "child"];
    let key = |item: &&str| ItemKey::text(item);
    let hierarchy = |item: &&str| {
        if *item == "branch" {
            TreeNode::parent(0)
        } else {
            TreeNode::leaf(1)
        }
    };
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for (state, selected, reference, cell_flags) in [
            ("rest", false, None, StateFlags::empty()),
            ("focused", false, Some(FOCUSED), StateFlags::FOCUSED),
            ("focused_selected", true, Some(FOCUSED), StateFlags::FOCUSED),
            (
                "focused_pressed",
                false,
                Some(FOCUSED_PRESSED),
                StateFlags::FOCUSED.union(StateFlags::PRESSED),
            ),
        ] {
            let mut st = TreeState::new();
            st.set_cursor(0, key(&items[0]));
            st.expand_all();
            let buf = painted(theme.clone(), reference.map(|s| (TREE, s)), |ui, _area| {
                Tree::new(TREE)
                    .cursor_selected(selected)
                    .key(key)
                    .node(&hierarchy)
                    .draw(ui, Rect::new(0, 1, 24, 3), &st, &items);
            });
            let cell = cell_of(&buf, 0, 1);
            let ctx = format!("{level:?} TREE {state}");
            assert_bar(&cell, cell_flags.contains(StateFlags::FOCUSED), &ctx);
            if !cell_flags.contains(StateFlags::FOCUSED) {
                continue;
            }
            let helper = helper_fg(
                &theme,
                FocusBarPolicy::RowKept,
                cell_flags,
                cell.bg.expect("row surface"),
                false,
            );
            if level == ColorLevel::Ansi16 && selected {
                assert_ne!(
                    cell.fg, helper,
                    "{ctx}: the focused selected bar ink diverges from the helper, so the recipe stays"
                );
            } else {
                assert_eq!(
                    cell.fg, helper,
                    "{ctx}: the frozen bar ink equals the helper ink, so the state could adopt"
                );
            }
        }
    }
}

#[test]
fn steps_row_gutters_match_the_helper_cell_in_every_probed_state() {
    let base = Theme::junie();
    let items = ["first", "second"];
    let key = |item: &&str| ItemKey::text(item);
    let step_state = |_: &&str| StepState::Queued;
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for (state, navigable, reference, cell_flags) in [
            ("rest", false, None, StateFlags::empty()),
            ("focused", true, Some(FOCUSED), StateFlags::FOCUSED),
            (
                "focused_pressed",
                true,
                Some(FOCUSED_PRESSED),
                StateFlags::FOCUSED.union(StateFlags::PRESSED),
            ),
        ] {
            let mut st = StepsState::default();
            st.set_cursor(0, key(&items[0]));
            let rail = if navigable {
                Steps::navigable(STEPS)
            } else {
                Steps::new(STEPS)
            };
            let buf = painted(theme.clone(), reference.map(|s| (STEPS, s)), |ui, _area| {
                rail.key(key)
                    .step(&step_state)
                    .draw(ui, Rect::new(0, 1, 24, 3), &st, &items);
            });
            let cell = cell_of(&buf, 0, 1);
            let ctx = format!("{level:?} STEPS {state}");
            assert_bar(&cell, cell_flags.contains(StateFlags::FOCUSED), &ctx);
            if cell_flags.contains(StateFlags::FOCUSED) {
                assert_eq!(
                    cell.fg,
                    helper_fg(
                        &theme,
                        FocusBarPolicy::RowKept,
                        cell_flags,
                        cell.bg.expect("row surface"),
                        false,
                    ),
                    "{ctx}: the frozen bar ink equals the helper ink, so the state could adopt"
                );
            }
        }
    }
}

/// The state the probes cannot place the cursor in, pinned at the resolver:
/// the downgrade rules blank the bar glyph but leave the focus ink in the
/// style, and the site's blank fill stamps it onto the reserved cell.
/// `GutterCell::Inherit` stamps nothing, so this stamp is what blocks the
/// `LIST`/`TREE`/`STEPS` sites from adopting the helpers outright.
#[test]
fn list_focused_disabled_blanks_still_stamp_the_focus_ink() {
    let base = Theme::junie();
    let focused_disabled = StateFlags::FOCUSED | StateFlags::DISABLED;
    for level in LEVELS {
        let theme = base.clone().for_level(level);
        for (family, policy) in [
            (Family::LIST, FocusBarPolicy::RowBlanked),
            (Family::TREE, FocusBarPolicy::RowKept),
            (Family::STEPS, FocusBarPolicy::RowKept),
        ] {
            let r = resolver(&theme, family, Part::GUTTER, focused_disabled);
            let bar = matches!(
                r.glyph,
                termrock_theme::Slot::Set(termrock_theme::GlyphRole::FocusBar)
            );
            if bar {
                continue; // kept-bar families at the colour tiers: no blank stamp
            }
            let ctx = format!("{level:?} {family:?} focused+disabled");
            assert!(
                r.style.as_style().fg.is_some(),
                "{ctx}: the frozen blank stamps the focus ink"
            );
            assert!(
                helper_blank_stamps_nothing(&theme, policy, focused_disabled),
                "{ctx}: the helper stamps nothing, so converting the site would drop the stamp"
            );
        }
    }
}
