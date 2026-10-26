use showcase_ui::{App, PageId};
use termrock::{
    ColorLevel, Family, Id, ItemKey, KeyCode, Part, PartRef, Rect, StateFlags, Surface, Theme,
    Variant,
};
use termrock_test_support::Harness;

const TEXTAREAS_TABS: Id = Id::root("showcase_ui::pages::textareas::page_tabs");
const TEXTAREAS_EDIT_ENTRY: Id = Id::root("showcase_ui::pages::textareas::option.edit_entry");
const TEXTAREAS_HEADING_CASES: [(u16, u16, &str); 16] = [
    (100, 30, "16"),
    (100, 30, "256"),
    (100, 30, "mono"),
    (100, 30, "truecolor"),
    (120, 40, "16"),
    (120, 40, "256"),
    (120, 40, "mono"),
    (120, 40, "truecolor"),
    (160, 50, "16"),
    (160, 50, "256"),
    (160, 50, "mono"),
    (160, 50, "truecolor"),
    (80, 24, "16"),
    (80, 24, "256"),
    (80, 24, "mono"),
    (80, 24, "truecolor"),
];

#[test]
fn all_page_headings_match_the_reference_capture_matrix() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = include_str!("fixtures/holla-page-headings.tsv");
    assert_eq!(fixture.lines().count(), 352);
    compare_reference_rows(fixture, 2)
}

#[test]
fn all_shell_headers_match_reference_captures_and_visibility_thresholds()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = include_str!("fixtures/holla-shell-headers.tsv");
    assert_eq!(fixture.lines().count(), 355);
    compare_reference_rows(fixture, 0)
}

fn compare_reference_rows(fixture: &str, row: usize) -> Result<(), Box<dyn std::error::Error>> {
    let mut textareas_cases = Vec::new();
    for line in fixture.lines() {
        let mut fields = line.splitn(5, '\t');
        let name = fields.next().ok_or("missing page")?;
        let width = fields.next().ok_or("missing width")?.parse()?;
        let height = fields.next().ok_or("missing height")?.parse()?;
        let color = fields.next().ok_or("missing capability")?;
        let expected = fields.next().ok_or("missing reference row")?;
        let level = match color {
            "truecolor" => ColorLevel::TrueColor,
            "256" => ColorLevel::Ansi256,
            "16" => ColorLevel::Ansi16,
            "mono" => ColorLevel::Mono,
            _ => return Err("unknown reference capability".into()),
        };
        let page = PageId::from_name(name).ok_or("unknown reference page")?;
        if row == 2 && name == "textareas" {
            assert_textareas_heading_contract(width, height, color, level)?;
            textareas_cases.push((width, height, color));
        } else {
            let h =
                Harness::new(App::with_page(page), Theme::junie(), width, height).with_color(level);
            assert_eq!(
                h.text().lines().nth(row).unwrap_or_default().trim_end(),
                expected,
                "{name} {width}x{height} {color} row {row}"
            );
        }
    }
    if row == 2 {
        assert_eq!(
            textareas_cases.as_slice(),
            TEXTAREAS_HEADING_CASES.as_slice(),
            "every declared TextAreas size/color case has a native heading contract"
        );
    }
    Ok(())
}

fn textareas_main_area(width: u16, height: u16) -> Option<Rect> {
    match (width, height) {
        (80, 24) => Some(Rect::new(21, 2, 59, 20)),
        (100, 30) => Some(Rect::new(21, 2, 79, 26)),
        (120, 40) => Some(Rect::new(26, 2, 94, 36)),
        (160, 50) => Some(Rect::new(26, 2, 134, 46)),
        _ => None,
    }
}

fn textareas_heading_contract(
    width: u16,
    height: u16,
) -> Option<(Rect, [(&'static str, Rect); 3])> {
    let main = textareas_main_area(width, height)?;
    let strip = Rect::new(
        main.x.saturating_add(12),
        main.y,
        main.width.saturating_sub(12),
        1,
    );
    let tabs = [
        ("Playground", Rect::new(strip.x, strip.y, 12, 1)),
        (
            "Options",
            Rect::new(strip.x.saturating_add(13), strip.y, 9, 1),
        ),
        (
            "States",
            Rect::new(strip.x.saturating_add(23), strip.y, 8, 1),
        ),
    ];
    Some((main, tabs))
}

fn assert_textareas_heading_contract(
    width: u16,
    height: u16,
    color: &str,
    level: ColorLevel,
) -> Result<(), Box<dyn std::error::Error>> {
    let (main, expected_tabs) =
        textareas_heading_contract(width, height).ok_or("unreviewed TextAreas heading geometry")?;
    let strip = Rect::new(
        main.x.saturating_add(12),
        main.y,
        main.width.saturating_sub(12),
        1,
    );
    let mut h = Harness::new(
        App::with_page(PageId::TextAreas),
        Theme::junie(),
        width,
        height,
    )
    .with_color(level);
    let theme = Theme::junie().downgrade(level);
    let context = format!("TextAreas {width}x{height} {color}");

    assert_eq!(h.app().page(), PageId::TextAreas, "{context} route");
    assert_eq!(
        h.area_of(TEXTAREAS_TABS),
        Some(strip),
        "{context} native tabs owner area"
    );
    assert_eq!(
        cell_text(&h, main.x, main.y, "Text areas".len() as u16),
        "Text areas",
        "{context} title begins the declared page heading row"
    );
    assert_eq!(
        h.cell(main.x.saturating_add(11), main.y).symbol(),
        " ",
        "{context} retains one title-to-tabs separator cell"
    );
    assert_eq!(strip.x, main.x.saturating_add(12));
    assert_eq!(strip.y, main.y);
    assert_eq!(strip.height, 1);
    assert!(
        strip.right() <= main.right(),
        "{context} strip is page-clipped"
    );

    assert!(
        h.area_of(TEXTAREAS_EDIT_ENTRY).is_none(),
        "{context} starts in Playground with Options controls retired"
    );
    for (index, (label, expected_area)) in expected_tabs.iter().enumerate() {
        let part = PartRef::item(Part::TAB, ItemKey::index(index));
        assert_eq!(
            h.area_of_part(TEXTAREAS_TABS, part),
            Some(*expected_area),
            "{context} tab {label} geometry"
        );
        assert_eq!(
            expected_area.height, 1,
            "{context} tab {label} stays on heading row"
        );
        assert_eq!(
            expected_area.intersection(strip),
            *expected_area,
            "{context} tab {label} is clipped"
        );
        assert_eq!(
            cell_text(
                &h,
                expected_area.x.saturating_add(1),
                expected_area.y,
                label.len() as u16,
            ),
            *label,
            "{context} tab {index} label and order"
        );
    }
    assert_eq!(
        h.area_of_part(TEXTAREAS_TABS, PartRef::item(Part::TAB, ItemKey::index(3)),),
        None,
        "{context} registers exactly the three declared native tabs"
    );
    for (_, area) in expected_tabs.iter().take(2) {
        assert_eq!(
            h.cell(area.right(), strip.y).symbol(),
            " ",
            "{context} native tabs retain their one-cell separator"
        );
    }

    for (index, (label, area)) in expected_tabs.iter().enumerate() {
        let flags = if index == 0 {
            StateFlags::ACTIVE
        } else {
            StateFlags::empty()
        };
        assert_native_tab_ink(&h, &theme, *area, label, flags, &context);
    }

    assert!(
        h.tab_to(TEXTAREAS_TABS),
        "{context} tabs enter the focus ring"
    );
    let focused = h.state_of(TEXTAREAS_TABS);
    assert!(
        focused.contains(StateFlags::FOCUSED),
        "{context} tabs are focused"
    );
    assert!(
        focused.contains(StateFlags::FOCUS_VISIBLE),
        "{context} keyboard focus is visible"
    );
    for (index, (label, area)) in expected_tabs.iter().enumerate() {
        let flags = if index == 0 {
            StateFlags::ACTIVE | StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE
        } else {
            StateFlags::empty()
        };
        assert_native_tab_ink(&h, &theme, *area, label, flags, &context);
    }

    let _ = h.key(KeyCode::Char('2'));
    assert_eq!(
        h.focus(),
        Some(TEXTAREAS_TABS),
        "{context} keeps Tabs focus"
    );
    assert!(
        h.state_of(TEXTAREAS_TABS)
            .contains(StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE),
        "{context} focused Tabs state survives selection"
    );
    assert_eq!(
        h.area_of(TEXTAREAS_TABS),
        Some(strip),
        "{context} selection preserves the native tabs owner geometry"
    );
    assert!(
        h.area_of(TEXTAREAS_EDIT_ENTRY).is_some(),
        "{context} selecting Options reveals its native edit-entry control"
    );
    for (index, (label, area)) in expected_tabs.iter().enumerate() {
        let flags = if index == 1 {
            StateFlags::ACTIVE | StateFlags::FOCUSED | StateFlags::FOCUS_VISIBLE
        } else {
            StateFlags::empty()
        };
        assert_native_tab_ink(&h, &theme, *area, label, flags, &context);
        assert_eq!(
            h.area_of_part(
                TEXTAREAS_TABS,
                PartRef::item(Part::TAB, ItemKey::index(index))
            ),
            Some(*area),
            "{context} selection preserves registered tab geometry for {label}"
        );
        assert_eq!(
            cell_text(&h, area.x.saturating_add(1), area.y, label.len() as u16,),
            *label,
            "{context} selection preserves label {label}"
        );
    }
    Ok(())
}

fn assert_native_tab_ink(
    h: &Harness<App>,
    theme: &Theme,
    area: Rect,
    label: &str,
    flags: StateFlags,
    context: &str,
) {
    let expected = theme.resolve(
        Family::TABS,
        Variant::DEFAULT,
        Part::TAB,
        flags,
        Surface::Canvas,
    );
    for x in area.x.saturating_add(1)..area.x.saturating_add(1 + label.len() as u16) {
        assert_eq!(
            Some(h.cell(x, area.y).fg),
            expected.style.fg,
            "{context} {label} theme foreground at column {x} with {flags:?}"
        );
    }
}

fn cell_text(h: &Harness<App>, x: u16, y: u16, width: u16) -> String {
    let mut text = String::new();
    for column in x..x.saturating_add(width) {
        text.push_str(h.cell(column, y).symbol());
    }
    text
}
