//! Owning EMPTY parts must style the actual shared readiness cells.
use junie_tui::{
    App, CellRef, Column, ColumnKey, Cx, EmptyState, Family, FilterList, FilterListState, Grid,
    GridModel, GridState, Id, Item, ItemKey, KeyCode, List, ListState, Modifier, Part, Rect,
    Response, Role, Select, SelectState, StylePatch, Theme, Tree, TreeState, Ui,
};
use junie_tui_testing::Harness;
use ratatui_core::style::Color;
#[global_allocator]
static GLOBAL: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
const OWNER: Id = Id::root("empty.owner");
const AREA: Rect = Rect::new(0, 0, 24, 9);
const EMPTY: EmptyState<'static> = EmptyState::Empty {
    title: "T",
    hint: Some("H"),
};
const PARTS: [(Part, StylePatch); 1] = [(
    Part::EMPTY,
    StylePatch::new()
        .set_fg(Role::Custom(Color::Red))
        .set_bg(Role::Custom(Color::Blue))
        .add(Modifier::ITALIC),
)];
const COLS: [Column<'static>; 1] = [Column::new(ColumnKey::num(1), "column")];
struct NoRows;
impl GridModel for NoRows {
    fn row_count(&self) -> usize {
        0
    }
    fn row_key(&self, _: usize) -> ItemKey {
        ItemKey::num(0)
    }
    fn cell(&self, _: usize, _: usize) -> Option<CellRef<'_>> {
        None
    }
}
#[derive(Default)]
struct Page {
    kind: u8,
    list: ListState,
    filter: FilterListState,
    select: SelectState,
    grid: GridState,
    tree: TreeState,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        match self.kind {
            0 => List::new(OWNER)
                .empty(EMPTY)
                .patch_part(&PARTS)
                .update(cx, &mut self.list, &[] as &[&str])
                .erase(),
            1 => FilterList::new(OWNER)
                .empty(EMPTY)
                .patch_part(&PARTS)
                .update(cx, &mut self.filter, &[] as &[Item<'_>])
                .erase(),
            2 => Select::new(OWNER)
                .empty(EMPTY)
                .patch_part(&PARTS)
                .update(cx, &mut self.select, &[] as &[&str])
                .erase(),
            3 => Grid::new(OWNER, &COLS)
                .empty(EMPTY)
                .patch_part(&PARTS)
                .update(cx, &mut self.grid, &NoRows)
                .erase(),
            _ => Tree::new(OWNER)
                .empty(EMPTY)
                .patch_part(&PARTS)
                .update(cx, &mut self.tree, &[] as &[&str])
                .erase(),
        }
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let sentinel = ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Custom(Color::Yellow))
                .set_bg(Role::Custom(Color::Green)),
        );
        ui.paint_str(Rect::new(31, 11, 1, 1), "S", sentinel);
        match self.kind {
            0 => {
                List::new(OWNER).empty(EMPTY).patch_part(&PARTS).draw(
                    ui,
                    AREA,
                    &self.list,
                    &[] as &[&str],
                );
            }
            1 => {
                FilterList::new(OWNER).empty(EMPTY).patch_part(&PARTS).draw(
                    ui,
                    AREA,
                    &self.filter,
                    &[] as &[Item<'_>],
                );
            }
            2 => {
                Select::new(OWNER).empty(EMPTY).patch_part(&PARTS).draw(
                    ui,
                    Rect::new(0, 0, 24, 1),
                    &self.select,
                    &[] as &[&str],
                );
            }
            3 => {
                Grid::new(OWNER, &COLS)
                    .empty(EMPTY)
                    .patch_part(&PARTS)
                    .draw(ui, AREA, &self.grid, &NoRows);
            }
            _ => {
                Tree::new(OWNER).empty(EMPTY).patch_part(&PARTS).draw(
                    ui,
                    AREA,
                    &self.tree,
                    &[] as &[&str],
                );
            }
        }
        List::new(Id::root("empty.other"))
            .empty(EmptyState::Empty {
                title: "Q",
                hint: None,
            })
            .draw(
                ui,
                Rect::new(25, 0, 7, 9),
                &ListState::default(),
                &[] as &[&str],
            );
    }
}
fn check(kind: u8, child: bool) {
    let theme = if child {
        Theme::junie().override_family(Family::EMPTY, |r| {
            r.part(Part::TITLE).base(
                StylePatch::new()
                    .set_fg(Role::Custom(Color::Cyan))
                    .add(Modifier::BOLD),
            );
        })
    } else {
        Theme::junie()
    };
    let mut h = Harness::new(
        Page {
            kind,
            ..Page::default()
        },
        theme,
        32,
        12,
    );
    if kind == 2 {
        let _ = h.key(KeyCode::Enter);
    }
    let mut title = None;
    let mut painted_blank = false;
    let mut other = None;
    for y in 0..12 {
        for x in 0..32 {
            let c = h.cell(x, y);
            if c.symbol() == "T" {
                title = Some((c.fg, c.bg, c.modifier));
            }
            if c.symbol() == "Q" {
                other = Some((c.fg, c.bg, c.modifier));
            }
            if c.symbol() == " " && c.bg == Color::Blue && c.modifier.contains(Modifier::ITALIC) {
                painted_blank = true;
            }
        }
    }
    assert_eq!(
        title.map(|(fg, bg, _)| (fg, bg)),
        Some((if child { Color::Cyan } else { Color::Red }, Color::Blue)),
        "owning EMPTY {kind}, child {child}"
    );
    assert!(
        title.is_some_and(|(_, _, m)| m.contains(Modifier::ITALIC)),
        "owning modifier {kind}"
    );
    if child {
        assert!(title.is_some_and(|(_, _, m)| m.contains(Modifier::BOLD)));
    }
    assert!(
        painted_blank,
        "owning empty background reaches blank cells {kind}"
    );
    assert_eq!(
        other.map(|(fg, _, _)| fg),
        Some(if child {
            Color::Cyan
        } else {
            Color::Rgb(128, 128, 128)
        }),
        "other instance"
    );
    assert!(
        other.is_some_and(|(_, bg, m)| bg != Color::Blue && !m.contains(Modifier::ITALIC)),
        "instance patch leaked"
    );
    assert_eq!(
        (
            h.cell(31, 11).symbol(),
            h.cell(31, 11).fg,
            h.cell(31, 11).bg
        ),
        ("S", Color::Yellow, Color::Green)
    );
}
#[test]
fn list_empty_inherits_owner() {
    check(0, false);
    check(0, true);
}
#[test]
fn filter_empty_inherits_owner() {
    check(1, false);
    check(1, true);
}
#[test]
fn select_empty_inherits_owner() {
    check(2, false);
    check(2, true);
}
#[test]
fn grid_empty_inherits_owner() {
    check(3, false);
    check(3, true);
}
#[test]
fn tree_empty_inherits_owner() {
    check(4, false);
    check(4, true);
}

#[test]
fn standalone_empty_cells_match_pre_inheritance_reference() {
    use junie_tui::{ColorLevel, Empty};
    use junie_tui_testing::Scene;
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for state in [
            EMPTY,
            EmptyState::Loading { label: "L" },
            EmptyState::Partial {
                loaded: 1,
                total: junie_tui::RowTotal::Exact(2),
                hint: "P",
            },
            EmptyState::Error {
                message: "E",
                detail: Some("D"),
            },
        ] {
            for standalone in [false, true] {
                let mut scene =
                    Scene::new("standalone_empty_reference", Theme::junie(), level, 24, 7);
                scene.draw(|ui, _| {
                    if standalone {
                        Empty::new(OWNER, state).draw(ui, Rect::new(0, 0, 24, 7));
                    } else {
                        state.draw(ui, Rect::new(0, 0, 24, 7), 0);
                    }
                });
                for cell in scene.buffer().content() {
                    for byte in format!("{cell:?}").bytes() {
                        fingerprint =
                            (fingerprint ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
                    }
                }
            }
        }
    }
    // Exact bytes captured from immutable db53dd5 before EMPTY inheritance.
    assert_eq!(fingerprint, 10_278_960_817_768_212_803);
}

#[test]
fn child_set_clear_reset_state_and_scopes_keep_their_precedence() {
    use junie_tui::{ColorLevel, Overlay, OverlayRule, StateFlags, Variant};
    use junie_tui_testing::Scene;
    static RULES: [OverlayRule; 1] = [(
        Family::EMPTY,
        Variant::DEFAULT,
        Part::TITLE,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Custom(Color::Green)),
    )];
    for child in [
        StylePatch::new()
            .set_fg(Role::Custom(Color::Cyan))
            .set_bg(Role::Custom(Color::Magenta)),
        StylePatch::new().clear_fg().clear_bg(),
        StylePatch::new()
            .set_fg(Role::Custom(Color::Reset))
            .set_bg(Role::Custom(Color::Reset)),
    ] {
        for error in [false, true] {
            let theme = Theme::junie().override_family(Family::EMPTY, |r| {
                r.part(Part::TITLE)
                    .base(child.add(Modifier::BOLD).remove(Modifier::ITALIC))
                    .when(
                        StateFlags::ERROR,
                        StylePatch::new().set_fg(Role::Custom(Color::Yellow)),
                    );
            });
            let mut scene = Scene::new("empty_precedence", theme, ColorLevel::TrueColor, 24, 7);
            let state = if error {
                EmptyState::Error {
                    message: "T",
                    detail: Some("H"),
                }
            } else {
                EMPTY
            };
            scene.draw(|ui, _| {
                let owner = ui.paint_patch(&PARTS[0].1);
                state.draw_inherited(ui, Rect::new(0, 0, 24, 3), 0, owner);
                ui.with_overlay(&Overlay::new(&RULES), |ui| {
                    state.draw_inherited(ui, Rect::new(0, 4, 24, 3), 0, owner);
                });
            });
            for (y, scoped) in [(0, false), (4, true)] {
                let title = scene
                    .buffer()
                    .content()
                    .iter()
                    .enumerate()
                    .find(|(i, c)| *i / 24 == y && c.symbol() == "T")
                    .map(|(_, c)| c);
                let explicit_set =
                    matches!(child.fg, junie_tui::Slot::Set(Role::Custom(Color::Cyan)));
                let fg = if scoped {
                    Color::Green
                } else if error {
                    Color::Yellow
                } else if explicit_set {
                    Color::Cyan
                } else {
                    Color::Red
                };
                let bg = if explicit_set {
                    Color::Magenta
                } else {
                    Color::Blue
                };
                assert_eq!(
                    title.map(|c| (c.fg, c.bg)),
                    Some((fg, bg)),
                    "state {error}, scope {scoped}, patch {child:?}"
                );
                assert!(title.is_some_and(|c| c.modifier.contains(Modifier::BOLD)
                    && !c.modifier.contains(Modifier::ITALIC)));
            }
        }
    }
}

#[test]
fn child_tones_preserve_parent_background_origin_without_allocating() {
    use junie_tui::{ColorLevel, FgStep};
    use junie_tui_testing::{
        Scene,
        perf::{allocs, bytes},
    };
    let mut scene = Scene::new(
        "empty_provenance",
        Theme::junie(),
        ColorLevel::TrueColor,
        24,
        3,
    );
    let state = EmptyState::Error {
        message: "T",
        detail: Some("H"),
    };
    let paint = |ui: &mut Ui<'_>| {
        let owner = ui.paint_patch(
            &StylePatch::new()
                .set_bg(Role::HoverSurface)
                .set_fg(Role::Fg(FgStep::Ghost)),
        );
        state.draw_inherited(ui, Rect::new(0, 0, 24, 3), 0, owner);
    };
    scene.draw(|ui, _| paint(ui));
    scene.draw(|ui, _| {
        let before = (allocs(), bytes());
        paint(ui);
        assert_eq!(
            (allocs(), bytes()),
            before,
            "inherited resolution and paint allocate nothing"
        );
        ui.dim_layer(Rect::new(0, 0, 24, 3), 1);
    });
    assert!(
        !scene.buffer().content().iter().any(|c| c.symbol() == "T"),
        "Ghost title retains owner origin"
    );
    for symbol in [
        "H",
        Theme::junie()
            .design
            .glyphs
            .get(junie_tui::GlyphRole::Error),
    ] {
        let cell = scene
            .buffer()
            .content()
            .iter()
            .find(|c| c.symbol() == symbol);
        assert_eq!(
            cell.map(|c| c.bg),
            Some(Color::Rgb(24, 24, 27)),
            "child tone {symbol}"
        );
    }
}
