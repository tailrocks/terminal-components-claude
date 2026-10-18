use junie_tui::core::event::Key;
use junie_tui::core::hit::Axis;
use junie_tui::core::id::WidgetId;
use junie_tui::core::keymap::NavKeys;
use junie_tui::core::list::{NavProfile, Navigable};
use junie_tui::widgets::chips::{Chip, ChipBar};
use junie_tui::widgets::choice::RadioGroup;
use junie_tui::widgets::completion::Completion;
use junie_tui::widgets::grid::{CellKind, CellValue, ColumnSpec, DataGrid, GridRows, RowTotal};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::menu::{ContextMenu, MenuBar, MenuItem};
use junie_tui::widgets::option_list::Item;
use junie_tui::widgets::picker::Picker;
use junie_tui::widgets::props::{Prop, PropsList};
use junie_tui::widgets::select::Select;
use junie_tui::widgets::steps::{Step, StepRail};
use junie_tui::widgets::table::{Cell, Column, DataTable};
use junie_tui::widgets::tabs::Tabs;
use junie_tui::widgets::tree::{TreeNode, TreeView};
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Constraint, Rect};

const LABELS: [&str; 5] = ["alpha", "bravo", "charlie", "delta", "echo"];
const DISABLED: [usize; 2] = [1, 4];

fn key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

trait Probe {
    fn press(&mut self, k: &Key);
    fn at(&self) -> usize;
    fn enabled(&self, i: usize) -> bool;
    fn axis(&self) -> Axis;
    fn wraps(&self) -> bool;
}

struct P<W> {
    w: W,
    on_key: fn(&mut W, &Key),
}

impl<W: Navigable> Probe for P<W> {
    fn press(&mut self, k: &Key) {
        (self.on_key)(&mut self.w, k);
    }
    fn at(&self) -> usize {
        self.w.cursor().index
    }
    fn enabled(&self, i: usize) -> bool {
        self.w.enabled(i)
    }
    fn axis(&self) -> Axis {
        self.w.cursor().axis
    }
    fn wraps(&self) -> bool {
        self.w.cursor().wrap
    }
}

fn probe<W: Navigable + 'static>(w: W, on_key: fn(&mut W, &Key)) -> Box<dyn Probe> {
    Box::new(P { w, on_key })
}

fn id(s: &str) -> WidgetId {
    WidgetId::of(s)
}

fn widgets() -> Vec<(&'static str, bool, bool, Box<dyn Probe>)> {
    let d = |i: usize| DISABLED.contains(&i);

    let mut completion = Completion::new(id("c")).nav(NavKeys::DESKTOP);
    completion.open(
        LABELS.iter().map(|l| Item::new(*l).glyph("T")).collect(),
        Rect::new(0, 0, 10, 1),
        0,
    );
    let mut picker = Picker::new(id("p"), "Pick");

    picker.set_searchable(false);
    picker.set_items(
        LABELS
            .iter()
            .enumerate()
            .map(|(i, l)| Item::new(*l).disabled(d(i)))
            .collect(),
    );
    let mut select = Select::new(id("s"), "S", &LABELS, 0);
    select.on_key(&key(KeyCode::Enter));
    let table = DataTable::new(
        id("t"),
        vec![Column::new("name", Constraint::Length(10))],
        LABELS.iter().map(|l| vec![Cell::new(*l)]).collect(),
    );
    let mut grid = DataGrid::new(id("g"), vec![ColumnSpec::new("name", CellKind::Text)]);
    grid.set_rows(GridRows {
        rows: LABELS
            .iter()
            .map(|l| vec![CellValue::Text((*l).into())])
            .collect(),
        total: RowTotal::Exact(5),
        more: false,
    });
    let mut chips = ChipBar::new(id("chips"));
    chips.chips = LABELS.iter().map(|l| Chip::new(l)).collect();

    chips.add_label = None;
    let tree = TreeView::new(
        id("tree"),
        LABELS
            .iter()
            .enumerate()
            .map(|(i, l)| {
                if d(i) {
                    TreeNode::note(l)
                } else {
                    TreeNode::leaf(l)
                }
            })
            .collect(),
    );
    vec![
        (
            "ListBox",
            true,
            true,
            probe(
                ListBox::new(
                    id("l"),
                    LABELS
                        .iter()
                        .enumerate()
                        .map(|(i, l)| ListItem::new(l).disabled(d(i)))
                        .collect(),
                    SelectMode::Single,
                ),
                |w, k| {
                    w.on_key(k);
                },
            ),
        ),
        (
            "TreeView",
            true,
            true,
            probe(tree, |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "ContextMenu",
            true,
            true,
            probe(
                ContextMenu::new(
                    id("m"),
                    LABELS
                        .iter()
                        .enumerate()
                        .map(|(i, l)| MenuItem::new(*l).disabled(d(i)))
                        .collect(),
                )
                .skip_disabled(true),
                |w, k| {
                    w.on_key(k);
                },
            ),
        ),
        (
            "Picker",
            true,
            true,
            probe(picker, |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "PropsList",
            false,
            true,
            probe(
                PropsList::new(
                    id("props"),
                    LABELS.iter().map(|l| Prop::new(l, "v")).collect(),
                ),
                |w, k| {
                    w.on_key(k);
                },
            ),
        ),
        (
            "StepRail",
            false,
            true,
            probe(
                StepRail::new(id("steps"), LABELS.iter().map(|l| Step::new(l)).collect())
                    .selectable(true),
                |w, k| {
                    w.on_key(k);
                },
            ),
        ),
        (
            "Completion",
            false,
            false,
            probe(completion, |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "Select (open)",
            false,
            true,
            probe(select, |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "RadioGroup",
            false,
            true,
            probe(RadioGroup::new(id("r"), "R", &LABELS, 0), |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "DataTable",
            false,
            true,
            probe(table, |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "DataGrid",
            false,
            true,
            probe(grid, |w, k| {
                let mods = match k.code {
                    KeyCode::Home | KeyCode::End => KeyModifiers::CONTROL,
                    _ => k.mods,
                };
                w.on_key(&Key { code: k.code, mods });
            }),
        ),
        (
            "Tabs",
            false,
            true,
            probe(Tabs::new(id("tabs"), &LABELS), |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "ChipBar",
            false,
            true,
            probe(chips, |w, k| {
                w.on_key(k);
            }),
        ),
        (
            "MenuBar",
            false,
            true,
            probe(
                MenuBar::new(
                    id("bar"),
                    LABELS
                        .iter()
                        .map(|l| (*l, vec![MenuItem::new("x")]))
                        .collect(),
                ),
                |w, k| {
                    w.on_key(k);
                },
            ),
        ),
    ]
}

#[test]
fn every_navigable_walks_the_same_rows() {
    NavProfile::Desktop.install();

    let walk = |fwd: KeyCode, back: KeyCode, wrap: bool| {
        let (last, plain_last) = if wrap { (3, 4) } else { (0, 0) };
        let (first, plain_first) = if wrap { (0, 0) } else { (3, 4) };
        [
            (back, last, plain_last),
            (KeyCode::Home, 0, 0),
            (fwd, 2, 1),
            (fwd, 3, 2),
            (KeyCode::End, 3, 4),
            (fwd, first, plain_first),
            (KeyCode::Home, 0, 0),
            (back, last, plain_last),
        ]
    };
    for (name, disabled, _, mut w) in widgets() {
        let (fwd, back) = match w.axis() {
            Axis::Vertical => (KeyCode::Down, KeyCode::Up),
            Axis::Horizontal => (KeyCode::Right, KeyCode::Left),
        };
        assert_eq!(w.at(), 0, "{name} starts on the first row");
        let steps = walk(fwd, back, w.wraps());
        for (step, (code, with_disabled, plain)) in steps.into_iter().enumerate() {
            w.press(&key(code));
            let want = if disabled { with_disabled } else { plain };
            assert_eq!(w.at(), want, "{name}: step {step} ({code:?})");
            assert!(
                w.enabled(w.at()),
                "{name}: step {step} stopped on a disabled row"
            );
        }
    }
}

#[test]
fn every_navigable_ignores_the_vim_letters_by_default_and_takes_them_under_vim() {
    for (profile, moves) in [(NavProfile::Desktop, false), (NavProfile::Vim, true)] {
        profile.install();
        for (name, _, letters, mut w) in widgets() {
            if !letters {
                continue;
            }
            let letter = match w.axis() {
                Axis::Vertical => 'j',
                Axis::Horizontal => 'l',
            };
            w.press(&key(KeyCode::Char(letter)));
            assert_eq!(
                w.at() != 0,
                moves,
                "{name} under {profile:?}: `{letter}` moved: {}",
                w.at()
            );
        }
    }
    NavProfile::Desktop.install();
}
