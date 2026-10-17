//! Per-widget documentation (plan §5): `docs/components/<widget>.md` and
//! its index are generated from the widgets themselves, so a page cannot
//! drift from the code. Each documented widget gives a [`WidgetDoc`]
//! (purpose, role, standards, knobs and its mouse table) and is built
//! under the Desktop and Vim profiles to read its live [`Bindings`] (the
//! keyboard table) and [`HasHints`] (hints by state).
//!
//! `cargo xtask docs components` rewrites the pages; the default test
//! `docs_components_fresh` fails when a page differs from what the
//! generator writes. S8b documents the widgets that have `Bindings`; later
//! steps add theirs (S15 makes a `WidgetDoc` a `Component` requirement).

use crate::core::hints::{Binding, Bindings, HasHints, HintCx};
use crate::core::id::WidgetId;
use crate::core::list::NavProfile;
use crate::widgets::keyhint::Hint;

/// Which override layer a knob sits in (AGENTS.md "Every behaviour
/// remains overridable").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnobLayer {
    /// App-wide policy or profile ([`NavProfile`], the theme).
    Profile,
    /// A per-widget field and its builder.
    Widget,
    /// The widget's key map (user remapping).
    Keymap,
}

impl KnobLayer {
    const fn label(self) -> &'static str {
        match self {
            KnobLayer::Profile => "profile",
            KnobLayer::Widget => "widget",
            KnobLayer::Keymap => "key map",
        }
    }
}

/// One setting of a widget.
#[derive(Debug, Clone, Copy)]
pub struct Knob {
    pub name: &'static str,
    pub default: &'static str,
    pub layer: KnobLayer,
    /// The standard the default follows (empty: none named).
    pub standard: &'static str,
}

/// One row of a widget's mouse table: what each gesture does on `target`.
#[derive(Debug, Clone, Copy)]
pub struct MouseRow {
    pub target: &'static str,
    pub click: &'static str,
    pub double: &'static str,
    pub secondary: &'static str,
    pub drag: &'static str,
    pub wheel: &'static str,
}

/// What a widget's page says beyond its live keys and hints.
#[derive(Debug, Clone, Copy)]
pub struct WidgetDoc {
    /// The type name (`ListBox`) and the page's file stem (`list_box`).
    pub name: &'static str,
    pub file: &'static str,
    /// The WAI-ARIA role or pattern it follows (`listbox`).
    pub role: &'static str,
    pub purpose: &'static str,
    /// `(name, url)` of each standard it follows.
    pub standards: &'static [(&'static str, &'static str)],
    pub knobs: &'static [Knob],
    pub mouse: &'static [MouseRow],
}

/// A documented widget: its doc and, per profile, its live key rows and
/// its hints in each state.
pub struct Documented {
    pub doc: &'static WidgetDoc,
    /// The widget's key rows when built under `profile`.
    pub bindings: fn(NavProfile) -> Vec<Binding>,
    /// `(state, hints)` of the widget built under `profile`.
    pub states: fn(NavProfile) -> Vec<(&'static str, Vec<Hint>)>,
}

const ID: WidgetId = WidgetId::of("docs");

/// Build under `profile`, restoring Desktop after.
fn under<T>(profile: NavProfile, build: impl FnOnce() -> T) -> T {
    let before = NavProfile::current();
    profile.install();
    let w = build();
    before.install();
    w
}

fn hints_of(w: &dyn HasHints) -> Vec<Hint> {
    w.hints(&HintCx::default())
}

/// Every documented widget, in index order.
pub fn documented() -> Vec<Documented> {
    use crate::widgets::grid::{CellKind, ColumnSpec, DataGrid};
    use crate::widgets::list::{ListBox, ListItem, SelectMode};
    use crate::widgets::select::Select;
    use crate::widgets::table::{Column, DataTable};
    use crate::widgets::tabs::Tabs;
    use crate::widgets::tree::{TreeNode, TreeView};
    use ratatui::layout::Constraint;

    fn list(mode: SelectMode) -> ListBox {
        ListBox::new(
            ID,
            vec![ListItem::new("Alpha"), ListItem::new("Beta")],
            mode,
        )
    }
    fn tree() -> TreeView {
        TreeView::new(ID, vec![TreeNode::leaf("a"), TreeNode::leaf("b")])
    }
    fn table() -> DataTable {
        DataTable::new(ID, vec![Column::new("Name", Constraint::Fill(1))], vec![])
    }
    fn grid() -> DataGrid {
        DataGrid::new(ID, vec![ColumnSpec::new("name", CellKind::Text)])
    }
    fn select() -> Select {
        Select::new(ID, "Kind", &["One", "Two"], 0)
    }
    vec![
        Documented {
            doc: &docs::LIST_BOX,
            bindings: |p| {
                let mut v = under(p, || list(SelectMode::Multi)).bindings();
                v.dedup();
                v
            },
            states: |p| {
                vec![
                    (
                        "single selection",
                        hints_of(&under(p, || list(SelectMode::Single))),
                    ),
                    (
                        "multiple selection",
                        hints_of(&under(p, || list(SelectMode::Multi))),
                    ),
                ]
            },
        },
        Documented {
            doc: &docs::TREE_VIEW,
            bindings: |p| under(p, tree).bindings(),
            states: |p| vec![("focused", hints_of(&under(p, tree)))],
        },
        Documented {
            doc: &docs::SELECT,
            bindings: |p| under(p, select).bindings(),
            states: |p| {
                let mut open = under(p, select);
                open.open = true;
                vec![
                    ("closed", hints_of(&under(p, select))),
                    ("open", hints_of(&open)),
                ]
            },
        },
        Documented {
            doc: &docs::TABS,
            bindings: |p| under(p, || Tabs::new(ID, &["One", "Two"])).bindings(),
            states: |p| {
                vec![(
                    "focused",
                    hints_of(&under(p, || Tabs::new(ID, &["One", "Two"]))),
                )]
            },
        },
        Documented {
            doc: &docs::DATA_TABLE,
            bindings: |p| under(p, table).bindings(),
            states: |p| vec![("focused", hints_of(&under(p, table)))],
        },
        Documented {
            doc: &docs::DATA_GRID,
            bindings: |p| under(p, grid).bindings(),
            states: |p| vec![("focused, no rows", hints_of(&under(p, grid)))],
        },
    ]
}

fn cell(s: &str) -> String {
    s.replace('|', "\\|")
}

/// The markdown page of `d`.
pub fn page(d: &Documented) -> String {
    let doc = d.doc;
    let mut out = String::new();
    out.push_str(&format!(
        "<!-- Generated by `cargo xtask docs components` from `src/docs.rs`; do not edit. -->\n\n# {}\n\n",
        doc.name
    ));
    out.push_str(&format!(
        "## Purpose and role\n\n{}\n\nRole: `{}`.\n\n",
        doc.purpose, doc.role
    ));
    // keyboard
    out.push_str("## Keyboard\n\n| Key (Desktop) | Key (Vim) | Action |\n|---|---|---|\n");
    let desk = (d.bindings)(NavProfile::Desktop);
    let vim = (d.bindings)(NavProfile::Vim);
    let mut actions: Vec<String> = vec![];
    for b in desk.iter().chain(&vim) {
        let a = b.action.to_string();
        if !actions.contains(&a) {
            actions.push(a);
        }
    }
    let keys_for = |rows: &[Binding], a: &str| {
        let ks: Vec<String> = rows
            .iter()
            .filter(|b| b.action == a)
            .map(|b| format!("`{}`", cell(&b.label)))
            .collect();
        if ks.is_empty() {
            "—".to_owned()
        } else {
            ks.join(" ")
        }
    };
    for a in &actions {
        out.push_str(&format!(
            "| {} | {} | {} |\n",
            keys_for(&desk, a),
            keys_for(&vim, a),
            cell(a)
        ));
    }
    // mouse
    out.push_str("\n## Mouse\n\n| Target | Click | Double click | Secondary click | Drag | Wheel |\n|---|---|---|---|---|---|\n");
    for m in doc.mouse {
        out.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} |\n",
            cell(m.target),
            cell(m.click),
            cell(m.double),
            cell(m.secondary),
            cell(m.drag),
            cell(m.wheel)
        ));
    }
    // hints
    out.push_str("\n## Hints by state\n\n| State | Desktop | Vim |\n|---|---|---|\n");
    let line = |h: &[Hint]| {
        let s: Vec<String> = h
            .iter()
            .map(|h| format!("`{}` {}", cell(h.key), cell(&h.action)))
            .collect();
        if s.is_empty() {
            "—".to_owned()
        } else {
            s.join(" · ")
        }
    };
    let ds = (d.states)(NavProfile::Desktop);
    let vs = (d.states)(NavProfile::Vim);
    for ((state, dh), (_, vh)) in ds.iter().zip(&vs) {
        out.push_str(&format!("| {} | {} | {} |\n", state, line(dh), line(vh)));
    }
    // knobs
    out.push_str("\n## Knobs\n\n| Knob | Default | Layer | Standard |\n|---|---|---|---|\n");
    for k in doc.knobs {
        out.push_str(&format!(
            "| `{}` | {} | {} | {} |\n",
            k.name,
            cell(k.default),
            k.layer.label(),
            if k.standard.is_empty() {
                "—"
            } else {
                k.standard
            }
        ));
    }
    // paths
    out.push_str("\n## Keyboard-only path\n\n");
    for a in &actions {
        out.push_str(&format!("- {}: {}\n", a, keys_for(&desk, a)));
    }
    out.push_str("\n## Mouse-first path\n\n");
    for m in doc.mouse {
        out.push_str(&format!(
            "- {}: click {}; double click {}; wheel {}.\n",
            m.target, m.click, m.double, m.wheel
        ));
    }
    out.push_str("\n## Standards\n\n");
    for (name, url) in doc.standards {
        out.push_str(&format!("- [{name}]({url})\n"));
    }
    out
}

/// `docs/components/README.md`: the index of roles and standards.
pub fn index(all: &[Documented]) -> String {
    let mut out = String::from(
        "<!-- Generated by `cargo xtask docs components` from `src/docs.rs`; do not edit. -->\n\n# Components\n\nOne page per widget, generated from its live key maps, hints and defaults (plan §5).\n\n| Widget | Role | Standards |\n|---|---|---|\n",
    );
    for d in all {
        let std: Vec<&str> = d.doc.standards.iter().map(|(n, _)| *n).collect();
        out.push_str(&format!(
            "| [{}]({}.md) | `{}` | {} |\n",
            d.doc.name,
            d.doc.file,
            d.doc.role,
            std.join(", ")
        ));
    }
    out
}

/// Every generated file as `(path under docs/components, contents)`.
pub fn files() -> Vec<(String, String)> {
    let all = documented();
    let mut v: Vec<(String, String)> = all
        .iter()
        .map(|d| (format!("{}.md", d.doc.file), page(d)))
        .collect();
    v.push(("README.md".to_owned(), index(&all)));
    v
}

/// The widgets' [`WidgetDoc`]s.
pub mod docs {
    use super::{Knob, KnobLayer, MouseRow, WidgetDoc};

    const APG_LISTBOX: (&str, &str) = (
        "WAI-ARIA APG: Listbox",
        "https://www.w3.org/WAI/ARIA/apg/patterns/listbox/",
    );
    const APG_TREE: (&str, &str) = (
        "WAI-ARIA APG: Tree View",
        "https://www.w3.org/WAI/ARIA/apg/patterns/treeview/",
    );
    const APG_TABS: (&str, &str) = (
        "WAI-ARIA APG: Tabs",
        "https://www.w3.org/WAI/ARIA/apg/patterns/tabs/",
    );
    const APG_GRID: (&str, &str) = (
        "WAI-ARIA APG: Grid",
        "https://www.w3.org/WAI/ARIA/apg/patterns/grid/",
    );
    const APG_COMBO: (&str, &str) = (
        "WAI-ARIA APG: Combobox (select-only)",
        "https://www.w3.org/WAI/ARIA/apg/patterns/combobox/examples/combobox-select-only/",
    );
    const WCAG_214: (&str, &str) = (
        "WCAG 2.2 SC 2.1.4 Character Key Shortcuts",
        "https://www.w3.org/WAI/WCAG22/Understanding/character-key-shortcuts.html",
    );
    const HIG: (&str, &str) = (
        "Apple HIG: Lists and tables",
        "https://developer.apple.com/design/human-interface-guidelines/lists-and-tables",
    );

    const NAV: Knob = Knob {
        name: "nav",
        default: "the NavProfile's keys (Desktop: arrows, PgUp/PgDn, Home/End)",
        layer: KnobLayer::Profile,
        standard: "APG keyboard interaction",
    };
    const TYPE_AHEAD: Knob = Knob {
        name: "type_ahead",
        default: "on under Desktop, off under Vim",
        layer: KnobLayer::Profile,
        standard: "APG type-ahead; WCAG 2.1.4",
    };
    const KEYMAP: Knob = Knob {
        name: "keymap",
        default: "the command enum's `default_keymap()`",
        layer: KnobLayer::Keymap,
        standard: "WCAG 2.1.4 (remappable)",
    };
    const CLICK: Knob = Knob {
        name: "click",
        default: "`ClickPolicy::DATA`: one click selects, a double click activates",
        layer: KnobLayer::Widget,
        standard: "macOS lists (SI14)",
    };
    const ROWS: MouseRow = MouseRow {
        target: "row",
        click: "moves the cursor (selects)",
        double: "activates the row",
        secondary: "selects the row, opens its context menu",
        drag: "—",
        wheel: "scrolls the rows",
    };
    const BAR: MouseRow = MouseRow {
        target: "scrollbar",
        click: "pages toward the click (`TrackClick::Page`)",
        double: "—",
        secondary: "—",
        drag: "the thumb scrolls",
        wheel: "scrolls",
    };

    pub const LIST_BOX: WidgetDoc = WidgetDoc {
        name: "ListBox",
        file: "list_box",
        role: "listbox",
        purpose: "A scrollable list of rows with one cursor; single or multiple selection.",
        standards: &[APG_LISTBOX, HIG, WCAG_214],
        knobs: &[NAV, TYPE_AHEAD, KEYMAP, CLICK],
        mouse: &[
            ROWS,
            MouseRow {
                target: "check mark (multiple selection)",
                click: "toggles the row",
                double: "—",
                secondary: "—",
                drag: "—",
                wheel: "scrolls the rows",
            },
            BAR,
        ],
    };
    pub const TREE_VIEW: WidgetDoc = WidgetDoc {
        name: "TreeView",
        file: "tree_view",
        role: "tree",
        purpose: "A hierarchy of foldable branches with one cursor.",
        standards: &[APG_TREE, WCAG_214],
        knobs: &[NAV, TYPE_AHEAD, KEYMAP, CLICK],
        mouse: &[
            ROWS,
            MouseRow {
                target: "fold glyph",
                click: "folds or unfolds the branch",
                double: "—",
                secondary: "—",
                drag: "—",
                wheel: "scrolls the rows",
            },
            BAR,
        ],
    };
    pub const SELECT: WidgetDoc = WidgetDoc {
        name: "Select",
        file: "select",
        role: "combobox (select-only)",
        purpose: "A closed field that opens a list of options to choose one.",
        standards: &[APG_COMBO, WCAG_214],
        knobs: &[
            NAV,
            TYPE_AHEAD,
            Knob {
                name: "dismiss",
                default: "Esc and an outside click close the list",
                layer: KnobLayer::Widget,
                standard: "APG combobox",
            },
        ],
        mouse: &[
            MouseRow {
                target: "field",
                click: "opens or closes the list",
                double: "—",
                secondary: "—",
                drag: "—",
                wheel: "—",
            },
            MouseRow {
                target: "option",
                click: "chooses it and closes",
                double: "—",
                secondary: "—",
                drag: "—",
                wheel: "scrolls the options",
            },
        ],
    };
    pub const TABS: WidgetDoc = WidgetDoc {
        name: "Tabs",
        file: "tabs",
        role: "tablist",
        purpose: "A strip of tabs, one selected; tabs may close and a `+` may add one.",
        standards: &[APG_TABS, WCAG_214],
        knobs: &[NAV, KEYMAP],
        mouse: &[
            MouseRow {
                target: "tab",
                click: "selects it",
                double: "—",
                secondary: "opens its menu",
                drag: "—",
                wheel: "—",
            },
            MouseRow {
                target: "close button",
                click: "closes the tab",
                double: "—",
                secondary: "—",
                drag: "—",
                wheel: "—",
            },
        ],
    };
    pub const DATA_TABLE: WidgetDoc = WidgetDoc {
        name: "DataTable",
        file: "data_table",
        role: "grid (read-mostly table)",
        purpose: "Rows of columns with a row cursor, sorting and optional inline edit.",
        standards: &[APG_GRID, HIG, WCAG_214],
        knobs: &[NAV, TYPE_AHEAD, KEYMAP, CLICK],
        mouse: &[
            ROWS,
            MouseRow {
                target: "header",
                click: "sorts by the column",
                double: "—",
                secondary: "—",
                drag: "—",
                wheel: "—",
            },
            BAR,
        ],
    };
    pub const DATA_GRID: WidgetDoc = WidgetDoc {
        name: "DataGrid",
        file: "data_grid",
        role: "grid",
        purpose: "A spreadsheet-like grid with a cell cursor, editing and pending changes.",
        standards: &[APG_GRID, WCAG_214],
        knobs: &[
            KEYMAP,
            CLICK,
            Knob {
                name: "editable",
                default: "on",
                layer: KnobLayer::Widget,
                standard: "APG grid",
            },
        ],
        mouse: &[
            MouseRow {
                target: "cell",
                click: "moves the cell cursor",
                double: "edits an editable cell",
                secondary: "moves the cursor, opens the cell menu",
                drag: "extends the cell range",
                wheel: "scrolls the rows",
            },
            BAR,
        ],
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/components")
    }

    /// The component pages match the generator; `UPDATE_DOCS=1` (what
    /// `cargo xtask docs components` sets) rewrites them.
    #[test]
    fn docs_components_fresh() {
        let dir = dir();
        let update = std::env::var_os("UPDATE_DOCS").is_some();
        if update {
            std::fs::create_dir_all(&dir).unwrap();
        }
        let mut stale = vec![];
        for (name, text) in files() {
            let path = dir.join(&name);
            if update {
                std::fs::write(&path, &text).unwrap();
            } else if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
                stale.push(name);
            }
        }
        assert!(
            stale.is_empty(),
            "docs/components is stale ({stale:?}): run `cargo xtask docs components`"
        );
    }

    /// The "no keyboard-only state" rule (plan §5, mouse model): every
    /// documented widget gives both audiences a complete path, so each
    /// page has key rows and mouse rows, and every widget that hints keys
    /// in some state also has a mouse row.
    #[test]
    fn every_documented_widget_has_a_keyboard_and_a_mouse_path() {
        for d in documented() {
            assert!(
                !(d.bindings)(NavProfile::Desktop).is_empty(),
                "{}: keys",
                d.doc.name
            );
            assert!(!d.doc.mouse.is_empty(), "{}: mouse rows", d.doc.name);
        }
    }
}
