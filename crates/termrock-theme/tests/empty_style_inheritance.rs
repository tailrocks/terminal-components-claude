use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_controls::Empty;
use termrock_core::{Id, ItemKey, KeyCode, Response};
use termrock_grid::{CellRef, Column, ColumnKey, Grid, GridModel, GridState};
use termrock_navigation::{FilterList, FilterListState, Item, List, ListState, Tree, TreeState};
use termrock_overlays::{Select, SelectState};
use termrock_runtime::{App, Cx, EmptyState, RowTotal, Ui};
use termrock_test_support::perf::Counting;
use termrock_test_support::{Harness, Scene};
use termrock_theme::{
    ColorLevel, Family, FgStep, GlyphRole, Modifier, Overlay, OverlayRule, Part, Role, Slot,
    StateFlags, StylePatch, Theme, Variant,
};
#[global_allocator]
static GLOBAL: Counting = Counting;
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
            Color::Rgb(138, 138, 138)
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

fn assert_projected_cell(
    scene: &Scene,
    level: ColorLevel,
    state: &str,
    standalone: bool,
    position: (u16, u16),
    expected: (&str, Color),
) {
    let (x, y) = position;
    let (symbol, fg) = expected;
    let context = format!("{level:?}/{state}/standalone={standalone} ({x},{y})");
    let cell = scene
        .buffer()
        .cell((x, y))
        .unwrap_or_else(|| panic!("{context}: expected rendered cell"));
    assert_eq!(cell.symbol(), symbol, "{context}: symbol");
    assert_eq!(cell.fg, fg, "{context}: foreground");
}

fn assert_frozen_empty_palette_cells(
    scene: &Scene,
    level: ColorLevel,
    state: &str,
    standalone: bool,
) {
    let expected: &[(u16, u16, &str, Color)] = match (level, state, standalone) {
        // TrueColor Danger: the error icon is red in both render paths.
        (ColorLevel::TrueColor, "Error", false) => &[(10, 0, "!", Color::Rgb(240, 93, 93))],
        (ColorLevel::TrueColor, "Error", true) => &[(10, 2, "!", Color::Rgb(240, 93, 93))],

        // ANSI 256 FgMuted titles/messages and FgFaint help use the frozen inks.
        (ColorLevel::Ansi256, "Empty", false) => &[
            (11, 0, "T", Color::Indexed(245)),
            (11, 2, "H", Color::Indexed(242)),
        ],
        (ColorLevel::Ansi256, "Empty", true) => &[
            (11, 2, "T", Color::Indexed(245)),
            (11, 4, "H", Color::Indexed(242)),
        ],
        (ColorLevel::Ansi256, "Loading", false) => &[(12, 0, "L", Color::Indexed(245))],
        (ColorLevel::Ansi256, "Loading", true) => &[(12, 3, "L", Color::Indexed(245))],
        (ColorLevel::Ansi256, "Partial", false) => &[(12, 0, "P", Color::Indexed(245))],
        (ColorLevel::Ansi256, "Partial", true) => &[(12, 3, "P", Color::Indexed(245))],
        (ColorLevel::Ansi256, "Error", false) => &[
            (12, 0, "E", Color::Indexed(245)),
            (11, 2, "D", Color::Indexed(242)),
            (10, 0, "!", Color::Indexed(203)),
        ],
        (ColorLevel::Ansi256, "Error", true) => &[
            (12, 2, "E", Color::Indexed(245)),
            (11, 4, "D", Color::Indexed(242)),
            (10, 2, "!", Color::Indexed(203)),
        ],

        // ANSI 16 FgFaint: Empty hints and Error details use Gray.
        (ColorLevel::Ansi16, "Empty", false) => &[(11, 2, "H", Color::Gray)],
        (ColorLevel::Ansi16, "Empty", true) => &[(11, 4, "H", Color::Gray)],
        (ColorLevel::Ansi16, "Error", false) => &[(11, 2, "D", Color::Gray)],
        (ColorLevel::Ansi16, "Error", true) => &[(11, 4, "D", Color::Gray)],
        _ => &[],
    };

    for &(x, y, symbol, fg) in expected {
        assert_projected_cell(scene, level, state, standalone, (x, y), (symbol, fg));
    }
}

#[test]
fn standalone_empty_cells_match_pre_inheritance_reference() {
    let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
    for level in [
        ColorLevel::TrueColor,
        ColorLevel::Ansi256,
        ColorLevel::Ansi16,
        ColorLevel::Mono,
    ] {
        for (state_name, state) in [
            ("Empty", EMPTY),
            ("Loading", EmptyState::Loading { label: "L" }),
            (
                "Partial",
                EmptyState::Partial {
                    loaded: 1,
                    total: RowTotal::Exact(2),
                    hint: "P",
                },
            ),
            (
                "Error",
                EmptyState::Error {
                    message: "E",
                    detail: Some("D"),
                },
            ),
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
                assert_frozen_empty_palette_cells(&scene, level, state_name, standalone);
                for cell in scene.buffer().content() {
                    for byte in format!("{cell:?}").bytes() {
                        fingerprint =
                            (fingerprint ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
                    }
                }
            }
        }
    }

    // Provenance: reviewed f08d20e0f3 rendered 2_278_838_586_319_248_431;
    // later 2ca2c7ad33 rendered 312_677_554_172_290_677. The 2ca-to-7a delta
    // is 20 foreground-only cells matching frozen tag 4169783/design f2f6eb7,
    // covered by the role/level assertions above.
    assert_eq!(fingerprint, 3_030_540_740_827_194_175);
}

#[test]
fn child_set_clear_reset_state_and_scopes_keep_their_precedence() {
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
                let explicit_set = matches!(child.fg, Slot::Set(Role::Custom(Color::Cyan)));
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
    use termrock_test_support::perf::{allocs, bytes};
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
    for symbol in ["H", Theme::junie().design.glyphs.get(GlyphRole::Error)] {
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
