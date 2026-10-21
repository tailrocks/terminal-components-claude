use termrock_core::{Id, ItemKey};
use termrock_navigation::{NavList, NavListState};
use termrock_runtime::Rect;
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Theme};

const ID: Id = Id::root("nav-list-row-policy");

#[derive(Clone, Copy)]
struct RowData {
    key: &'static str,
    label: &'static str,
}

fn key(item: &RowData) -> ItemKey {
    ItemKey::text(item.key)
}

fn draw(data: &[RowData], gap: Option<u16>, width: u16, height: u16) -> Scene {
    let mut scene = Scene::new(
        "nav-list-row-policy",
        Theme::junie(),
        ColorLevel::TrueColor,
        20,
        4,
    );
    scene.draw(|ui, _| {
        let list = NavList::new(ID).key(key).row(
            |item: &RowData, row: &mut termrock_runtime::RowUi<'_>| {
                row.label(item.label);
            },
        );
        let list = match gap {
            Some(gap) => list.content_gap(gap),
            None => list,
        };
        list.draw(
            ui,
            Rect::new(2, 1, width, height),
            &NavListState::new(),
            data,
        );
    });
    scene
}

#[test]
fn default_gap_preserves_existing_prefix_geometry_and_zero_gap_is_composable() {
    let items = [RowData {
        key: "stable-key",
        label: "Q",
    }];

    let default = draw(&items, None, 12, 1);
    let explicit_default = draw(&items, Some(1), 12, 1);
    let compact_gap = draw(&items, Some(0), 12, 1);

    assert_eq!(default.digest(), explicit_default.digest());
    assert_eq!(default.buffer().cell((5, 1)).unwrap().symbol(), " ");
    assert_eq!(default.buffer().cell((6, 1)).unwrap().symbol(), "Q");
    assert_eq!(compact_gap.buffer().cell((5, 1)).unwrap().symbol(), "Q");
    assert_eq!(compact_gap.buffer().cell((6, 1)).unwrap().symbol(), " ");
}

#[test]
fn gap_handles_empty_narrow_and_zero_sized_rows_without_panicking() {
    let items = [RowData {
        key: "wide",
        label: "世界e\u{301}🙂",
    }];

    for (gap, width, height) in [(0, 0, 0), (u16::MAX, 1, 1), (0, 3, 1), (2, 8, 1)] {
        let _ = draw(&items, Some(gap), width, height);
    }
    let empty = draw(&[], Some(0), 8, 1);
    assert!(empty.text().contains("                    "));
}
