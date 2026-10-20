#[path = "../examples/file_catalogue_master_detail.rs"]
mod example;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::blocks::MasterDetailPolicy;
use termrock::blocks::{MasterDetail, MasterDetailState};
use termrock::composition::{self, Group};
use termrock::controls::Button;
use termrock::{App, Cx, Id, LayerSpec, Response, Ui};
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
