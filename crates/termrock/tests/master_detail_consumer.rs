#[path = "../examples/file_catalogue_master_detail.rs"]
mod example;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui_core::layout::{Position, Rect};
use termrock::blocks::MasterDetailPolicy;
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
