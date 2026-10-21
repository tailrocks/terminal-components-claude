#[path = "../examples/file_catalogue_master_detail.rs"]
mod example;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock::blocks::MasterDetailPolicy;
use termrock::blocks::{MasterDetail, MasterDetailState};
use termrock::composition::{self, Group};
use termrock::controls::Button;
use termrock::{App, Cx, Id, LayerSpec, Part, PartRef, Response, Ui};
use termrock::{Input, Key, KeyCode, KeyModifiers, Mouse, MouseKind, Runtime, Theme};

fn present<T: termrock::App>(runtime: &mut Runtime<T>, area: Rect) {
    let prepared = runtime.prepare(area).expect("prepare frame");
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal
        .draw(|frame| {
            runtime
                .draw_prepared(prepared, frame)
                .expect("draw prepared frame");
        })
        .expect("present test frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("acknowledge frame");
}

fn capture_file_catalogue(
    policy: MasterDetailPolicy,
    area: Rect,
) -> (Buffer, Option<Position>, bool, Vec<Option<Rect>>) {
    let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
    let prepared = runtime.prepare(area).expect("prepare frame");
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal
        .draw(|frame| {
            runtime.draw_prepared(prepared, frame).expect("draw frame");
        })
        .expect("test backend draws frame");
    runtime
        .acknowledge_presented(prepared)
        .expect("acknowledge frame");
    let regions = [example::README, example::PREVIEW, example::SUMMARY]
        .into_iter()
        .map(|id| runtime.area_of(id))
        .collect();
    (
        terminal.backend().buffer().clone(),
        runtime.cursor(),
        runtime.focus_visible(),
        regions,
    )
}

#[test]
fn explicit_fixed_policy_keeps_default_complete_frames_and_control_regions() {
    let default = MasterDetailPolicy::default();
    let explicit = MasterDetailPolicy::default()
        .detail_share(34, 34, 48)
        .gap(2)
        .collapse_below(108)
        .summary_rows(1);
    for width in [0, 1, 59, 60, 99, 100, 104, 107, 108, 109, 120] {
        for height in [0, 2, 4, 10, 12] {
            let area = Rect::new(0, 0, width, height);
            assert_eq!(
                capture_file_catalogue(default, area),
                capture_file_catalogue(explicit, area),
                "fixed default frame and regions at {width}x{height}"
            );
        }
    }
}

#[test]
fn opt_in_master_ratio_uses_one_layout_across_breakpoints_and_survives_redraw() {
    let policy = MasterDetailPolicy::default()
        .resizable_master(34, 30, 40)
        .gap(2)
        .collapse_below(100)
        .summary_rows(6);

    for (width, expected_detail, expected_seam) in [
        (99, None, None),
        (100, Some((35, 65)), Some(34)),
        (101, Some((35, 66)), Some(34)),
        (102, Some((36, 66)), Some(35)),
    ] {
        let area = Rect::new(0, 0, width, 24);
        let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
        present(&mut runtime, area);
        if let Some(expected_detail) = expected_detail {
            assert_eq!(runtime.area_of(example::README).unwrap().x, 0);
            let detail = runtime.area_of(example::PREVIEW).unwrap();
            assert_eq!((detail.x, detail.width), expected_detail);
            let seam = runtime
                .area_of_part(example::WORKSPACE, PartRef::of(Part::SEAM))
                .expect("resizable master has a hit-testable seam");
            assert_eq!(seam.x, expected_seam.expect("split mode has a seam"));
        } else {
            assert!(runtime.area_of(example::PREVIEW).is_none());
            assert!(runtime.area_of(example::SUMMARY).is_some());
        }
    }

    for height in 0..=4 {
        let area = Rect::new(0, 0, 99, height);
        let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
        present(&mut runtime, area);
    }
    for (height, summary_y) in [(24, 18), (17, 12)] {
        let area = Rect::new(0, 0, 99, height);
        let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
        present(&mut runtime, area);
        assert_eq!(runtime.area_of(example::SUMMARY).unwrap().y, summary_y);
    }

    let wide = Rect::new(0, 0, 120, 24);
    let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
    present(&mut runtime, wide);
    assert_eq!(runtime.app().split_percent(), Some(34));
    let seam = runtime
        .area_of_part(example::WORKSPACE, PartRef::of(Part::SEAM))
        .expect("shared SplitPane registers the visible seam");
    let seam_point = Position::new(seam.x, seam.y);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Down,
            pos: seam_point,
            mods: KeyModifiers::NONE,
        }))
        .expect("press the painted seam");
    present(&mut runtime, wide);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Drag,
            pos: Position::new(seam.x.saturating_add(5), seam.y),
            mods: KeyModifiers::NONE,
        }))
        .expect("drag the captured seam");
    present(&mut runtime, wide);
    let _ = runtime
        .handle(Input::Mouse(Mouse {
            kind: MouseKind::Up,
            pos: Position::new(seam.x.saturating_add(5), seam.y),
            mods: KeyModifiers::NONE,
        }))
        .expect("release the captured seam");
    present(&mut runtime, wide);
    let resized = runtime
        .app()
        .split_percent()
        .expect("update initializes split state");
    assert_ne!(resized, 34);
    assert_eq!(
        runtime.app().last_layout_action(),
        Some(termrock::MasterDetailAction::Resized(resized))
    );

    let resized_area = Rect::new(0, 0, 150, 24);
    let _ = runtime
        .handle(Input::Resize(resized_area.width, resized_area.height))
        .expect("resize invalidates the old split layout");
    let basis_points = runtime
        .app()
        .split_basis_points()
        .expect("split ratio survives resize");
    let expected_seam_x =
        (u32::from(resized_area.width - 2) * u32::from(basis_points) / 10_000) as u16 + 1;
    let pending_pointer = Input::Mouse(Mouse {
        kind: MouseKind::Down,
        pos: Position::new(expected_seam_x, 2),
        mods: KeyModifiers::NONE,
    });
    let pending_pointer = runtime
        .handle(pending_pointer.clone())
        .expect_err("input waits for the resized layout to be presented")
        .into_input();
    let new_layout = expected_seam_x;
    present(&mut runtime, resized_area);
    let new_seam = runtime
        .area_of_part(example::WORKSPACE, PartRef::of(Part::SEAM))
        .expect("new layout publishes the new seam hit region");
    assert_eq!(new_seam.x, new_layout);
    let _ = runtime
        .handle(pending_pointer)
        .expect("replay pending input against the presented resized layout");
    present(&mut runtime, resized_area);

    present(&mut runtime, Rect::new(0, 0, 99, 24));
    present(&mut runtime, wide);
    assert_eq!(runtime.app().split_percent(), Some(resized));
}

const PURE_BLOCK: Id = Id::root("public.master-detail.pure");
const PURE_MASTER: Id = Id::root("public.master-detail.pure.master");
const PURE_DETAIL: Id = Id::root("public.master-detail.pure.detail");

struct MeasurementOnlyCatalogue {
    state: MasterDetailState,
    editor: termrock::editors::CodeEditorState,
}

impl App for MeasurementOnlyCatalogue {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let button = Button::new(PURE_MASTER, "Notebook");
        let editor = termrock::editors::CodeEditor::new(PURE_DETAIL, 1);
        let master = Group::row([composition::button(&button)]);
        let detail = Group::row([composition::code_editor(&editor, &self.editor)]);
        let policy = MasterDetailPolicy::default()
            .resizable_master(34, 30, 40)
            .gap(2)
            .collapse_below(100)
            .summary_rows(6);
        let block = MasterDetail::new(PURE_BLOCK, PURE_MASTER, PURE_DETAIL)
            .policy(policy)
            .master(&master)
            .detail(&detail);
        let _measured = block
            .measure(
                ui,
                &self.state,
                termrock::layout::Constraints::loose(ui.full().width, ui.full().height),
            )
            .expect("measure the consumer block without effects");
        block
            .show(ui, &self.state)
            .expect("draw the consumer block without state initialization");
    }
}

#[test]
fn facade_measure_and_draw_do_not_initialize_consumer_split_state() {
    let mut runtime = Runtime::new(
        MeasurementOnlyCatalogue {
            state: MasterDetailState::default(),
            editor: termrock::editors::CodeEditorState::new("body"),
        },
        Theme::junie(),
    );
    present(&mut runtime, Rect::new(0, 0, 120, 10));
    assert_eq!(runtime.app().state.split(), None);
}

const ALT_FIRST: Id = Id::root("independent-catalogue.document.first");
const ALT_SECOND: Id = Id::root("independent-catalogue.document.second");
const ALT_EMPTY: Id = Id::root("independent-catalogue.document.empty-label");
const ALT_DOCUMENTS: [example::CatalogueDocument; 3] = [
    example::CatalogueDocument {
        id: ALT_SECOND,
        name: "東京からの長い旅行記 — 二番目",
        summary: "旅行記 · 23 行",
        contents: "# 東京\n\n日本語の文書です。\n",
    },
    example::CatalogueDocument {
        id: ALT_FIRST,
        name: "résumé́.txt",
        summary: "Résumé · 4 lines",
        contents: "name = \"Zoë\"\nplace = \"Hà Nội\"\n",
    },
    example::CatalogueDocument {
        id: ALT_EMPTY,
        name: "",
        summary: "Empty label remains a stable row",
        contents: "",
    },
];

#[test]
fn second_public_catalogue_uses_unrelated_reordered_unicode_data_and_policy() {
    let policy = MasterDetailPolicy::default()
        .resizable_master(45, 18, 22)
        .gap(2)
        .collapse_below(100)
        .summary_rows(6);
    let area = Rect::new(0, 0, 120, 24);
    let mut runtime = Runtime::new(
        example::FileCatalogue::with_documents(policy, &ALT_DOCUMENTS),
        Theme::junie(),
    );
    present(&mut runtime, area);

    let first = runtime.area_of(ALT_SECOND).expect("reordered first row");
    let second = runtime.area_of(ALT_FIRST).expect("reordered second row");
    let empty = runtime
        .area_of(ALT_EMPTY)
        .expect("empty label keeps its own ID");
    assert!(first.y < second.y && second.y < empty.y);
    let detail = runtime
        .area_of(example::PREVIEW)
        .expect("detail editor is composed");
    assert_eq!((detail.x, detail.width), (55, 65));
    assert_eq!(
        runtime.app().selected_document(),
        "東京からの長い旅行記 — 二番目"
    );
    assert!(runtime.app().preview_text().contains("日本語"));
    assert_eq!(runtime.app().split_percent(), Some(45));
}

#[test]
fn public_master_detail_consumer_uses_live_document_groups_and_fixed_policy() {
    let policy = MasterDetailPolicy::default();
    for (width, expected_detail) in [(107, None), (108, Some(36)), (109, Some(37))] {
        let area = Rect::new(0, 0, width, 12);
        let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
        present(&mut runtime, area);
        let master = runtime
            .area_of(example::README)
            .expect("master file control");
        assert_eq!(master.x, 0);
        if let Some(expected_detail) = expected_detail {
            let detail = runtime.area_of(example::PREVIEW).expect("detail editor");
            assert_eq!(detail.width, expected_detail, "root width {width}");
            assert_eq!(detail.x, width - expected_detail);
            assert!(runtime.area_of(example::SUMMARY).is_none());
        } else {
            assert!(runtime.area_of(example::PREVIEW).is_none());
            assert!(runtime.area_of(example::SUMMARY).is_some());
        }
        assert_eq!(runtime.app().selected_document(), "README.md");
        assert!(!runtime.app().drawer_open());
    }
}

#[test]
fn public_master_detail_drawer_opens_closes_and_restores_focus_repeatedly() {
    let area = Rect::new(0, 0, 59, 10);
    let mut runtime = Runtime::new(example::FileCatalogue::default(), Theme::junie());
    present(&mut runtime, area);
    assert!(runtime.area_of(example::PREVIEW).is_none());
    assert!(runtime.area_of(example::SUMMARY).is_some());

    for _ in 0..2 {
        let file = runtime
            .area_of(example::MANIFEST)
            .expect("second document row");
        let point = Position::new(file.x, file.y);
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind: MouseKind::Down,
                pos: point,
                mods: KeyModifiers::NONE,
            }))
            .expect("press file row");
        present(&mut runtime, area);
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind: MouseKind::Up,
                pos: point,
                mods: KeyModifiers::NONE,
            }))
            .expect("activate file row");
        present(&mut runtime, area);

        assert_eq!(runtime.app().selected_document(), "manifest.toml");
        assert!(runtime.app().drawer_open());
        assert!(runtime.area_of(example::PREVIEW).is_some());
        assert_eq!(runtime.focus(), Some(example::PREVIEW));

        let wide = Rect::new(0, 0, 108, 10);
        present(&mut runtime, wide);
        let _ = runtime
            .handle(Input::Key(Key {
                code: KeyCode::Esc,
                mods: KeyModifiers::NONE,
            }))
            .expect("Escape in the wide layout is handled by the detail owner");
        present(&mut runtime, wide);
        assert!(runtime.app().drawer_open());

        // Resize back before Escape: the same input now dismisses the visible drawer.
        present(&mut runtime, area);

        let _ = runtime
            .handle(Input::Key(Key {
                code: KeyCode::Esc,
                mods: KeyModifiers::NONE,
            }))
            .expect("dismiss detail drawer");
        present(&mut runtime, area);
        assert!(!runtime.app().drawer_open());
        assert!(runtime.area_of(example::PREVIEW).is_none());
        assert_eq!(runtime.focus(), Some(example::README));
    }
}

#[test]
fn non_escape_input_is_still_delivered_to_the_detail_owner() {
    let area = Rect::new(0, 0, 59, 10);
    let mut runtime = Runtime::new(example::FileCatalogue::default(), Theme::junie());
    present(&mut runtime, area);

    let file = runtime
        .area_of(example::MANIFEST)
        .expect("second document row");
    let point = Position::new(file.x, file.y);
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind,
                pos: point,
                mods: KeyModifiers::NONE,
            }))
            .expect("select second document");
        present(&mut runtime, area);
    }
    assert_eq!(runtime.focus(), Some(example::PREVIEW));

    let editor = runtime
        .area_of(example::PREVIEW)
        .expect("detail editor area");
    let text_point = Position::new(editor.x.saturating_add(3), editor.y);
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind,
                pos: text_point,
                mods: KeyModifiers::NONE,
            }))
            .expect("place cursor in focused detail editor");
        present(&mut runtime, area);
    }

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }))
        .expect("enter editor insert mode");
    present(&mut runtime, area);

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Char('x'),
            mods: KeyModifiers::NONE,
        }))
        .expect("ordinary input reaches focused detail editor");
    assert!(runtime.app().preview_text().contains('x'));
    assert!(runtime.app().drawer_open());
}

#[test]
fn public_master_detail_handles_narrow_heights_and_zero_sized_area() {
    for height in 0..=4 {
        example::render_once(42, height);
    }
    example::render_once(0, 0);
}

#[test]
fn summary_reserves_the_requested_height_clamped_to_the_short_viewport() {
    let policy = MasterDetailPolicy::default()
        .collapse_below(60)
        .summary_rows(3);
    for height in 0..=4 {
        let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
        present(&mut runtime, Rect::new(0, 0, 59, height));
    }
    for (height, expected_y) in [(6, 4), (9, 6), (12, 9)] {
        let area = Rect::new(0, 0, 59, height);
        let mut runtime = Runtime::new(example::FileCatalogue::with_policy(policy), Theme::junie());
        present(&mut runtime, area);
        assert_eq!(
            runtime
                .area_of(example::SUMMARY)
                .expect("summary in usable height")
                .y,
            expected_y,
            "summary rows reserve fixed height at height={height}"
        );
    }
}

#[test]
fn nested_block_uses_its_clipped_width_and_scope_instead_of_terminal_viewport() {
    let area = Rect::new(0, 0, 120, 12);
    let mut runtime = Runtime::new(NestedCatalogue::default(), Theme::junie());
    present(&mut runtime, area);
    assert_eq!(runtime.area_of(NESTED_MASTER).unwrap().x, 20);
    assert!(runtime.area_of(NESTED_DETAIL_A).is_none());

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert_eq!(runtime.focus(), Some(NESTED_DETAIL_A));
    assert!(runtime.area_of(NESTED_WORKSPACE).is_none());

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert_eq!(runtime.focus(), Some(NESTED_DETAIL_B));

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::F(2),
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(runtime.app().state.drawer_open());
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(runtime.app().state.drawer_open(), "top modal owns Escape");

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert_eq!(runtime.focus(), Some(NESTED_SIBLING));

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(
        runtime.app().state.drawer_open(),
        "sibling focus is outside detail scope"
    );

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert_eq!(runtime.focus(), Some(NESTED_DETAIL_A));

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Tab,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert_eq!(runtime.focus(), Some(NESTED_DETAIL_B));

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(!runtime.app().state.drawer_open());
    assert_eq!(runtime.focus(), Some(NESTED_MASTER));
}

#[test]
fn editor_escape_precedes_the_scoped_drawer_escape() {
    let area = Rect::new(0, 0, 59, 10);
    let mut runtime = Runtime::new(example::FileCatalogue::default(), Theme::junie());
    present(&mut runtime, area);
    let file = runtime.area_of(example::MANIFEST).unwrap();
    for kind in [MouseKind::Down, MouseKind::Up] {
        let _ = runtime
            .handle(Input::Mouse(Mouse {
                kind,
                pos: Position::new(file.x, file.y),
                mods: KeyModifiers::NONE,
            }))
            .unwrap();
        present(&mut runtime, area);
    }
    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Enter,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(runtime.app().drawer_open());

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(
        runtime.app().drawer_open(),
        "editor consumes its first Escape"
    );

    let _ = runtime
        .handle(Input::Key(Key {
            code: KeyCode::Esc,
            mods: KeyModifiers::NONE,
        }))
        .unwrap();
    present(&mut runtime, area);
    assert!(!runtime.app().drawer_open());
}

const NESTED_WORKSPACE: Id = Id::root("test.nested.workspace");
const NESTED_MASTER: Id = Id::root("test.nested.master");
const NESTED_DETAIL_A: Id = Id::root("test.nested.detail.a");
const NESTED_DETAIL_B: Id = Id::root("test.nested.detail.b");
const NESTED_SIBLING: Id = Id::root("test.nested.sibling");
const NESTED_MODAL: Id = Id::root("test.nested.modal");

#[derive(Default)]
struct NestedCatalogue {
    state: MasterDetailState,
}

impl NestedCatalogue {
    fn with_block<R>(&self, f: impl FnOnce(&MasterDetail<'_>) -> R) -> R {
        let master_button = Button::new(NESTED_MASTER, "Open detail");
        let master = Group::column([composition::button(&master_button)]);
        let detail_a = Button::new(NESTED_DETAIL_A, "Detail A");
        let detail_b = Button::new(NESTED_DETAIL_B, "Detail B");
        let detail = Group::column([
            composition::button(&detail_a),
            composition::button(&detail_b),
        ]);
        let block = MasterDetail::new(NESTED_WORKSPACE, NESTED_MASTER, NESTED_DETAIL_A)
            .policy(MasterDetailPolicy::default().collapse_below(60));
        let block = block.master(&master).detail(&detail);
        f(&block)
    }
}

impl App for NestedCatalogue {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let block = MasterDetail::new(NESTED_WORKSPACE, NESTED_MASTER, NESTED_DETAIL_A)
            .policy(MasterDetailPolicy::default().collapse_below(60));
        let response = block.update(cx, &mut self.state);
        if response.is_consumed() {
            return response.erase();
        }
        if cx
            .intents(NESTED_DETAIL_B)
            .any(|intent| matches!(intent, termrock::Intent::Key(key) if key.code == KeyCode::F(2)))
        {
            cx.open_layer(NESTED_MODAL, LayerSpec::modal(NESTED_MODAL));
            return Response::consumed();
        }
        let _ = Button::new(NESTED_SIBLING, "Other panel").update(cx);
        let activated = Button::new(NESTED_MASTER, "Open detail").update(cx);
        if activated.activated() {
            return block.open_detail(cx, &mut self.state).erase();
        }
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = Rect::new(20, 0, 59, 12);
        let _ = self.with_block(|block| ui.with_area(area, |ui| block.show(ui, &self.state)));
        let sibling = Button::new(NESTED_SIBLING, "Other panel");
        let sibling = Group::row([composition::button(&sibling)]);
        let _ = ui.with_area(Rect::new(0, 11, 15, 1), |ui| sibling.show(ui));
        let _ = ui.layer_slot(NESTED_MODAL, |_slot| {});
    }
}
