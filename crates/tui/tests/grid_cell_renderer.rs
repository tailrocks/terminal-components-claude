//! External consumer proof for the borrowed Grid display painter.
use std::cell::RefCell;

use junie_tui::GridCell;
use junie_tui::{
    ActionKey, App, CellAction, CellRef, CellUi, Column, ColumnKey, Cx, EditIntent, GlyphRole,
    Grid, GridEditor, GridModel, GridState, Id, ItemKey, KeyCode, Modifier, Part, Rect, Response,
    Role, StateFlags, StylePatch, Theme, Ui,
};
use junie_tui_testing::Harness;
use ratatui_core::style::Color;

#[global_allocator]
static GLOBAL: junie_tui_testing::perf::Counting = junie_tui_testing::perf::Counting;
const ID: Id = Id::root("custom.grid");
const PARTS: [(Part, StylePatch); 1] = [(
    Part::CELL,
    StylePatch::new()
        .set_fg(Role::Custom(Color::Red))
        .set_bg(Role::Custom(Color::Blue))
        .add(Modifier::ITALIC),
)];

struct Model {
    keys: [u64; 3],
}
impl Model {
    #[expect(
        clippy::expect_used,
        reason = "test rejects out-of-range model callbacks"
    )]
    fn key(&self, row: usize) -> u64 {
        *self.keys.get(row).expect("Grid supplied valid model row")
    }
}
const ACTIONS: [CellAction; 1] = [CellAction::new(ActionKey::custom("custom.action"))];
impl GridModel for Model {
    fn row_count(&self) -> usize {
        3
    }
    fn row_key(&self, row: usize) -> ItemKey {
        ItemKey::num(self.key(row))
    }
    fn cell(&self, row: usize, col: usize) -> Option<CellRef<'_>> {
        if row == 2 && col == 1 {
            None
        } else {
            Some(CellRef::new("old"))
        }
    }
    fn actions(&self, _: usize, col: usize) -> &[CellAction] {
        if col == 1 { &ACTIONS } else { &[] }
    }
}
impl GridEditor for Model {
    fn edit_intent(&self, row: usize, _: usize) -> EditIntent<'_> {
        if self.key(row) == 30 {
            EditIntent::Refuse { reason: "denied" }
        } else {
            EditIntent::Inline { initial: "old" }
        }
    }
    fn apply_cycle(&mut self, _: usize, _: usize) {}
    fn commit_cell(&mut self, _: usize, _: usize, _: &str) -> Result<(), junie_tui::FieldError> {
        Ok(())
    }
    fn is_editable(&self, _: usize, _: usize) -> bool {
        true
    }
}
struct Page {
    state: GridState,
    model: Model,
    seen: RefCell<Vec<(ItemKey, ColumnKey, StateFlags)>>,
    long: bool,
    editable: bool,
}
impl Default for Page {
    fn default() -> Self {
        Self {
            state: GridState::default(),
            model: Model { keys: [10, 20, 30] },
            seen: RefCell::new(Vec::with_capacity(32)),
            long: false,
            editable: false,
        }
    }
}
fn columns() -> [Column<'static>; 2] {
    let mut columns = [
        Column::new(ColumnKey::num(1), "A"),
        Column::new(ColumnKey::num(2), "B"),
    ];
    for col in &mut columns {
        col.min_width = 8;
        col.max_width = 8;
        col.editable = true;
    }
    columns[0].prefix_glyph = Some(GlyphRole::Checked);
    columns
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if self.editable {
            Grid::new(ID, &columns())
                .patch_part(&PARTS)
                .update_editable(cx, &mut self.state, &mut self.model)
                .erase()
        } else {
            Grid::new(ID, &columns())
                .patch_part(&PARTS)
                .update(cx, &mut self.state, &self.model)
                .erase()
        }
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        self.seen.borrow_mut().clear();
        let custom = |cell: GridCell<'_>, out: &mut CellUi<'_>| {
            self.seen
                .borrow_mut()
                .push((cell.row_key, cell.column_key, cell.flags));
            assert_eq!(cell.style.fg, Some(Color::Red));
            if self.long {
                out.text("界界界界界界界界界界界界");
            } else if cell.column == 0 {
                out.money(self.model.key(cell.row) as i64);
            } else {
                out.text("NULL");
            }
        };
        Grid::new(ID, &columns())
            .patch_part(&PARTS)
            .cell(&custom)
            .draw(ui, Rect::new(1, 1, 23, 6), &self.state, &self.model);
        let sentinel = ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::Yellow)));
        ui.paint_str(Rect::new(25, 2, 1, 1), "S", sentinel);
    }
}

#[test]
fn borrowed_renderer_replaces_text_once_and_preserves_cell_overrides() {
    let theme = Theme::junie().override_family(junie_tui::Family::GRID, |recipe| {
        recipe
            .part(Part::CELL)
            .base(StylePatch::new().set_fg(Role::Custom(Color::Green)));
    });
    let h = Harness::new(Page::default(), theme, 28, 8);
    assert_eq!(h.app().seen.borrow().len(), 5);
    assert!(!h.text().contains("old"));
    assert!(h.text().contains("NULL"));
    for y in 0..8 {
        for x in 0..28 {
            let cell = h.cell(x, y);
            if matches!(cell.symbol(), "N" | "U" | "L" | ".") {
                assert_eq!(cell.fg, Color::Red);
                assert_eq!(cell.bg, Color::Blue);
                assert!(cell.modifier.contains(Modifier::ITALIC));
            }
        }
    }
}

#[test]
fn borrowed_renderer_tracks_logical_cursor_after_reordering() {
    let mut h = Harness::new(Page::default(), Theme::junie(), 28, 8);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Char(' '));
    let before = h.app().state.cursor();
    h.app_mut().model.keys.swap(0, 1);
    let _ = h.key(KeyCode::Null);
    assert_eq!(h.app().state.cursor(), before);
    assert!(before.is_some_and(|(key, _)| h.app().state.selected_rows().contains(key)));
    let active: Vec<_> = h
        .app()
        .seen
        .borrow()
        .iter()
        .filter(|(_, _, flags)| flags.contains(StateFlags::ACTIVE))
        .copied()
        .collect();
    assert_eq!(active.len(), 1);
    assert_eq!(active.first().map(|&(row, col, _)| (row, col)), before);
    assert!(
        active
            .first()
            .is_some_and(|(_, _, flags)| flags.contains(StateFlags::SELECTED))
    );
}

#[test]
fn custom_wide_text_cannot_overwrite_prefix_actions_or_neighbor_cells() {
    let ordinary = Harness::new(Page::default(), Theme::junie(), 28, 8);
    let custom = Harness::new(
        Page {
            long: true,
            ..Page::default()
        },
        Theme::junie(),
        28,
        8,
    );
    assert_eq!(custom.app().seen.borrow().len(), 5);
    assert_eq!(custom.cell(25, 2).symbol(), "S");
    let mut checked = 0;
    for y in 0..8 {
        for x in 0..28 {
            let original = ordinary.cell(x, y);
            if [
                Theme::junie().design.glyphs.get(GlyphRole::Checked),
                Theme::junie().design.glyphs.get(GlyphRole::FollowRef),
            ]
            .contains(&original.symbol())
            {
                assert_eq!(custom.cell(x, y), original);
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 5);
}

#[test]
fn custom_renderer_is_skipped_for_inline_editor_and_refusal_text() {
    let mut h = Harness::new(
        Page {
            editable: true,
            ..Page::default()
        },
        Theme::junie(),
        28,
        8,
    );
    let _ = h.key(KeyCode::F(2));
    assert_eq!(h.app().seen.borrow().len(), 4);
    assert!(h.text().contains("old"));
    let _ = h.key(KeyCode::Esc);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::Down);
    let _ = h.key(KeyCode::F(2));
    assert_eq!(h.app().seen.borrow().len(), 4);
    assert!(h.text().contains("denied"));
}

#[test]
fn warmed_custom_cell_draw_allocates_nothing() {
    let mut h = Harness::new(Page::default(), Theme::junie(), 28, 8);
    h.draw();
    let before = junie_tui_testing::perf::allocs();
    h.draw();
    assert_eq!(junie_tui_testing::perf::allocs() - before, 0);
}

#[test]
fn custom_cell_retains_semantic_roles_through_alignment_and_dimming() {
    use junie_tui::{Align, ColorLevel, FgStep, Surface};
    use junie_tui_testing::Scene;
    let patch = [(
        Part::CELL,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Ghost))
            .set_bg(Role::Surface(Surface::Overlay)),
    )];
    let mut scene = Scene::new("grid_roles", Theme::junie(), ColorLevel::TrueColor, 28, 8);
    scene.draw(|ui, _| {
        let custom = |_: GridCell<'_>, out: &mut CellUi<'_>| {
            out.text("G").align(Align::Right);
        };
        Grid::new(ID, &columns())
            .patch_part(&patch)
            .cell(&custom)
            .draw(
                ui,
                Rect::new(1, 1, 23, 6),
                &GridState::default(),
                &Model { keys: [10, 20, 30] },
            );
    });
    let positions: Vec<_> = (0..8)
        .flat_map(|y| (0..28).map(move |x| (x, y)))
        .filter(|&p| scene.buffer().cell(p).is_some_and(|c| c.symbol() == "G"))
        .collect();
    assert_eq!(positions.len(), 5);
    scene.draw(|ui, _| {
        let custom = |_: GridCell<'_>, out: &mut CellUi<'_>| {
            out.text("G").align(Align::Right);
        };
        Grid::new(ID, &columns())
            .patch_part(&patch)
            .cell(&custom)
            .draw(
                ui,
                Rect::new(1, 1, 23, 6),
                &GridState::default(),
                &Model { keys: [10, 20, 30] },
            );
        ui.dim_layer(Rect::new(0, 0, 28, 8), 1);
    });
    for p in positions {
        assert_eq!(
            scene.buffer().cell(p).map(|c| (c.symbol(), c.bg)),
            Some((" ", Color::Rgb(39, 39, 42)))
        );
    }
}
