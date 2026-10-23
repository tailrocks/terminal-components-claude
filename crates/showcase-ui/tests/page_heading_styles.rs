use std::collections::HashSet;

use ratatui::buffer::CellDiffOption;
use ratatui::layout::Position;
use ratatui::style::Style;
use showcase_demos::pages::{Page as DemoPage, progress::ProgressPage, settings::SettingsPage};
use showcase_ui::{App, PageId};
use termrock::{
    App as TermrockApp, Color, ColorLevel, Cx, Family, Modifier, Part, Rect, Response, Role,
    StateFlags, StylePatch, Surface, Theme, Ui, Variant, width,
};
use termrock_test_support::Harness;

struct DirectPage<P> {
    page: P,
    area: Rect,
}

impl<P: DemoPage> TermrockApp for DirectPage<P> {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let (buffer, _) = ui.raw();
        let full = *buffer.area();
        buffer.set_style(
            full,
            Style::default()
                .fg(Color::Rgb(201, 17, 33))
                .bg(Color::Rgb(21, 45, 79))
                .underline_color(Color::Rgb(101, 111, 121))
                .add_modifier(Modifier::BOLD | Modifier::ITALIC),
        );
        for pos in full.positions() {
            let cell = buffer.cell_mut(pos).expect("sentinel cell is in bounds");
            cell.set_symbol("X");
            cell.set_diff_option(CellDiffOption::AlwaysUpdate);
        }
        for pos in self.area.positions() {
            buffer
                .cell_mut(pos)
                .expect("page cell is in bounds")
                .reset();
        }
        self.page.draw(ui, self.area);
    }
}

#[derive(Clone, Copy)]
struct Heading {
    route: &'static str,
    page: PageId,
    title: &'static str,
    meta: &'static str,
}

const HEADINGS: &[Heading] = &[
    Heading {
        route: "chips",
        page: PageId::Chips,
        title: "Chips & selects",
        meta: "Removable chips, a popup select, and strips that drop what does not fit",
    },
    Heading {
        route: "chrome",
        page: PageId::Chrome,
        title: "Chrome",
        meta: "Brand lockup · menu bar with anchored menus · status bar planes and priorities · context menu · hint layers",
    },
    Heading {
        route: "dialogs",
        page: PageId::Dialogs,
        title: "Dialogs",
        meta: "Focus is trapped, the page dims, Esc always cancels",
    },
    Heading {
        route: "grid",
        page: PageId::Grid,
        title: "Data grid",
        meta: "Typed cells, a pending-change queue, paging and local sort",
    },
    Heading {
        route: "lists",
        page: PageId::Lists,
        title: "Lists",
        meta: "Single and multiple selection, disabled items, scrolling, empty state",
    },
    Heading {
        route: "overview",
        page: PageId::Overview,
        title: "Overview",
        meta: "Tokens and principles behind every component",
    },
    Heading {
        route: "panels",
        page: PageId::Panels,
        title: "Panels",
        meta: "Cards group; a frame only where a pane needs an edge; nothing boxed twice",
    },
    Heading {
        route: "pickers",
        page: PageId::Pickers,
        title: "Pickers",
        meta: "One modal list for files, tabs, levels, commands and branches: search, scope, status",
    },
    Heading {
        route: "progress",
        page: PageId::Progress,
        title: "Progress",
        meta: "Determinate, indeterminate, an aggregate tally, capacity meters",
    },
    Heading {
        route: "scrolling",
        page: PageId::Scrolling,
        title: "Scrolling",
        meta: "Wheel under the pointer, keys on the focused container, thumb shows where you are, edges fade toward hidden rows",
    },
    Heading {
        route: "settings",
        page: PageId::Settings,
        title: "Project settings",
        meta: "Composed: tabs, form, editable table, list, dialogs",
    },
    Heading {
        route: "sidebars",
        page: PageId::Sidebars,
        title: "Sidebars",
        meta: "Sections, current item, focus cursor, hover, collapsed mode; text first, no icons",
    },
    Heading {
        route: "tables",
        page: PageId::Tables,
        title: "Tables",
        meta: "Sort by header, hover rows, select with Enter, overflow scrolls",
    },
    Heading {
        route: "trees",
        page: PageId::Trees,
        title: "Trees",
        meta: "Indent carries hierarchy; the focus bar never moves",
    },
];

const LEVELS: [(ColorLevel, &str, Color, Color, Color); 4] = [
    (
        ColorLevel::TrueColor,
        "truecolor",
        Color::Rgb(255, 255, 255),
        Color::Rgb(138, 138, 138),
        Color::Rgb(0, 0, 0),
    ),
    (
        ColorLevel::Ansi256,
        "256",
        Color::Indexed(231),
        Color::Indexed(245),
        Color::Indexed(16),
    ),
    (
        ColorLevel::Ansi16,
        "16",
        Color::White,
        Color::Gray,
        Color::Black,
    ),
    (
        ColorLevel::Mono,
        "mono",
        Color::White,
        Color::Gray,
        Color::Black,
    ),
];

fn level(name: &str) -> Option<(ColorLevel, Color, Color, Color)> {
    LEVELS
        .iter()
        .find(|(_, label, _, _, _)| *label == name)
        .map(|(level, _, title, meta, canvas)| (*level, *title, *meta, *canvas))
}

fn heading(route: &str) -> Option<Heading> {
    HEADINGS.iter().copied().find(|item| item.route == route)
}

fn heading_origin<A: termrock::App>(harness: &Harness<A>, title: &str, meta: &str) -> (u16, u16) {
    let prefix: String = meta.chars().take(8).collect();
    let needle = format!("{title}  {prefix}");
    let row = harness.row(2);
    let byte = row
        .find(&needle)
        .unwrap_or_else(|| panic!("missing rendered heading {needle:?} in row {row:?}"));
    (width(&row[..byte]), 2)
}

fn assert_default_heading<A: termrock::App>(
    harness: &Harness<A>,
    title: &str,
    meta: &str,
    title_fg: Color,
    meta_fg: Color,
    canvas: Color,
    label: &str,
) {
    let (x, y) = heading_origin(harness, title, meta);
    let title_cell = harness.cell(x, y);
    assert_eq!(
        title_cell.symbol(),
        title.chars().next().unwrap().to_string(),
        "{label} title glyph"
    );
    assert_eq!(title_cell.fg, title_fg, "{label} title role");
    assert_eq!(title_cell.bg, canvas, "{label} active surface");
    assert_eq!(
        title_cell.underline_color,
        Color::Reset,
        "{label} title underline"
    );
    assert!(
        title_cell.modifier.contains(Modifier::BOLD),
        "{label} title weight"
    );

    let title_width = width(title);
    let meta_x = x.saturating_add(title_width).saturating_add(2);
    let gap_start = x.saturating_add(title_width);
    assert_eq!(
        harness.cell(gap_start, y).symbol(),
        " ",
        "{label} first title/meta gap"
    );
    assert_eq!(
        harness.cell(gap_start + 1, y).symbol(),
        " ",
        "{label} second title/meta gap"
    );
    let meta_cell = harness.cell(meta_x, y);
    assert_eq!(
        meta_cell.symbol(),
        meta.chars().next().unwrap().to_string(),
        "{label} metadata glyph"
    );
    assert_eq!(meta_cell.fg, meta_fg, "{label} metadata role");
    assert_eq!(meta_cell.bg, canvas, "{label} metadata surface");
    assert_eq!(
        meta_cell.underline_color,
        Color::Reset,
        "{label} metadata underline"
    );
    assert!(
        !meta_cell.modifier.contains(Modifier::BOLD),
        "{label} metadata weight"
    );
}

#[test]
fn real_page_headings_use_junie_roles_across_all_352_fixture_cases() {
    let fixture = include_str!("fixtures/holla-page-headings.tsv");
    assert_eq!(fixture.lines().count(), 352);
    let mut routes = HashSet::new();
    let mut sizes = HashSet::new();
    let mut levels = HashSet::new();
    let mut tested = 0usize;

    for line in fixture.lines() {
        let mut fields = line.splitn(5, '\t');
        let route = fields.next().expect("page route");
        let width: u16 = fields.next().expect("width").parse().expect("number");
        let height: u16 = fields.next().expect("height").parse().expect("number");
        let color = fields.next().expect("color level");
        let _historical_row = fields.next().expect("retained source row");
        routes.insert(route);
        sizes.insert((width, height));
        levels.insert(color);

        let Some(expected) = heading(route) else {
            continue;
        };
        let (level, title_fg, meta_fg, canvas) = level(color).expect("known color level");
        let harness = Harness::new(App::with_page(expected.page), Theme::junie(), width, height)
            .with_color(level);
        assert_default_heading(
            &harness,
            expected.title,
            expected.meta,
            title_fg,
            meta_fg,
            canvas,
            &format!("{} {width}x{height} {color}", expected.route),
        );
        if expected.page == PageId::Settings {
            assert_eq!(
                harness.cell(width - 1, 2).symbol(),
                " ",
                "settings keeps its trailing heading blank at {width}x{height} {color}"
            );
        }
        tested += 1;
    }

    assert_eq!(routes.len(), 22);
    assert_eq!(sizes.len(), 4);
    assert_eq!(levels.len(), 4);
    assert_eq!(tested, HEADINGS.len() * 4 * 4);
}

fn custom_heading_theme(level: ColorLevel) -> Theme {
    Theme::junie()
        .for_level(level)
        .override_family(Family::PANEL, |recipe| {
            recipe.part(Part::TITLE).base(
                StylePatch::new()
                    .set_fg(Role::Custom(Color::Rgb(1, 2, 3)))
                    .set_bg(Role::Custom(Color::Rgb(4, 5, 6)))
                    .set_underline(Role::Custom(Color::Rgb(7, 8, 9)))
                    .add(Modifier::BOLD),
            );
        })
        .override_family(Family::LIST, |recipe| {
            recipe.part(Part::META).base(
                StylePatch::new()
                    .set_fg(Role::Custom(Color::Rgb(10, 11, 12)))
                    .set_bg(Role::Custom(Color::Reset))
                    .set_underline(Role::Custom(Color::Rgb(13, 14, 15)))
                    .add(Modifier::ITALIC),
            );
        })
}

#[test]
fn real_pages_preserve_authored_part_channels_and_reset_surface_overrides() {
    for (level, label, _, _, canvas) in LEVELS {
        let theme = custom_heading_theme(level);
        let title_style = theme.resolve(
            Family::PANEL,
            Variant::DEFAULT,
            Part::TITLE,
            StateFlags::empty(),
            Surface::Canvas,
        );
        let meta_style = theme.resolve(
            Family::LIST,
            Variant::DEFAULT,
            Part::META,
            StateFlags::empty(),
            Surface::Canvas,
        );
        for expected in HEADINGS {
            let harness = Harness::new(App::with_page(expected.page), theme.clone(), 120, 40)
                .with_color(level);
            let (x, y) = heading_origin(&harness, expected.title, expected.meta);
            let title = harness.cell(x, y);
            assert_eq!(
                title.fg,
                title_style.style.fg.unwrap(),
                "{} {label} title fg",
                expected.route
            );
            assert_eq!(
                title.bg,
                title_style.style.bg.unwrap(),
                "{} {label} title bg",
                expected.route
            );
            assert_eq!(
                title.underline_color,
                title_style.style.underline_color.unwrap(),
                "{} {label} title underline",
                expected.route
            );
            assert!(
                title.modifier.contains(Modifier::BOLD),
                "{} {label} title modifier",
                expected.route
            );

            let meta_x = x + width(expected.title) + 2;
            let meta = harness.cell(meta_x, y);
            assert_eq!(
                meta.fg,
                meta_style.style.fg.unwrap(),
                "{} {label} meta fg",
                expected.route
            );
            assert_eq!(
                meta.bg,
                meta_style.style.bg.unwrap_or(canvas),
                "{} {label} Reset inherits active surface",
                expected.route
            );
            assert_eq!(
                meta.underline_color,
                meta_style.style.underline_color.unwrap(),
                "{} {label} meta underline",
                expected.route
            );
            assert!(
                meta.modifier.contains(Modifier::ITALIC),
                "{} {label} meta modifier",
                expected.route
            );
        }
    }
}

#[test]
fn paper_page_heading_uses_paper_text_and_canvas_roles() {
    let expected = heading("lists").unwrap();
    let harness = Harness::new(App::with_page(expected.page), Theme::paper(), 120, 40);
    let (x, y) = heading_origin(&harness, expected.title, expected.meta);
    let title = harness.cell(x, y);
    let meta = harness.cell(x + width(expected.title) + 2, y);
    assert_eq!(title.fg, Color::Rgb(27, 26, 23));
    assert_eq!(title.bg, Color::Rgb(251, 250, 248));
    assert_eq!(meta.fg, Color::Rgb(119, 113, 106));
    assert_eq!(meta.bg, Color::Rgb(251, 250, 248));
}

#[test]
fn narrow_chrome_heading_keeps_middle_dot_and_clips_with_styled_ellipsis() {
    let expected = heading("chrome").unwrap();
    let harness = Harness::new(App::with_page(expected.page), Theme::junie(), 80, 24);
    let (x, y) = heading_origin(&harness, expected.title, expected.meta);
    let meta_x = x + width(expected.title) + 2;
    let row = harness.row(y);
    let byte = row.find("Chrome  Brand").unwrap();
    let meta_byte = byte + width(expected.title) as usize + 2;
    let shown = row[meta_byte..].trim_end();
    assert!(shown.starts_with("Brand lockup ·"), "{shown:?}");
    assert!(shown.ends_with('…'), "{shown:?}");
    assert_eq!(harness.cell(78, y).symbol(), "…");
    assert_eq!(harness.cell(78, y).fg, Color::Rgb(138, 138, 138));
    assert_eq!(harness.cell(78, y).bg, Color::Rgb(0, 0, 0));
    assert_eq!(
        harness.cell(79, y).symbol(),
        " ",
        "trailing cell remains outside metadata clip"
    );
    assert_eq!(harness.cell(meta_x, y).symbol(), "B");
}

fn direct_page_harness<P: DemoPage>(
    page: P,
    theme: Theme,
    color: ColorLevel,
    area: Rect,
) -> Harness<DirectPage<P>> {
    let width = area.right().saturating_add(2);
    let height = area.bottom().saturating_add(2);
    Harness::new(DirectPage { page, area }, theme, width, height).with_color(color)
}

fn rect_contains_rect(outer: Rect, inner: Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

fn assert_outside_sentinels<A: TermrockApp>(harness: &Harness<A>, area: Rect, label: &str) {
    for y in 0..harness.buffer().area().height {
        for x in 0..harness.buffer().area().width {
            let pos = Position::new(x, y);
            if area.contains(pos) {
                continue;
            }
            let cell = harness.cell(x, y);
            assert_eq!(cell.symbol(), "X", "{label}: outside symbol at {x},{y}");
            assert_eq!(cell.fg, Color::Rgb(201, 17, 33), "{label}: outside fg");
            assert_eq!(cell.bg, Color::Rgb(21, 45, 79), "{label}: outside bg");
            assert_eq!(
                cell.underline_color,
                Color::Rgb(101, 111, 121),
                "{label}: outside underline"
            );
            assert_eq!(
                cell.modifier,
                Modifier::BOLD | Modifier::ITALIC,
                "{label}: outside modifiers"
            );
            assert_eq!(
                cell.diff_option,
                CellDiffOption::AlwaysUpdate,
                "{label}: outside diff option"
            );
        }
    }
}

fn assert_area_boundaries<A: TermrockApp>(harness: &mut Harness<A>, area: Rect, label: &str) {
    for entry in harness.ring().entries() {
        assert!(
            rect_contains_rect(area, entry.area),
            "{label}: focus entry {:?} escaped {:?}",
            entry.area,
            area
        );
    }
    if let Some(cursor) = harness.cursor() {
        assert!(area.contains(cursor), "{label}: cursor escaped {area:?}");
    }
    assert_outside_sentinels(harness, area, label);
    let bounds = *harness.buffer().area();
    let center = Position::new(
        area.x.saturating_add(area.width / 2),
        area.y.saturating_add(area.height / 2),
    );
    let candidates = [
        area.x.checked_sub(1).map(|x| Position::new(x, center.y)),
        (area.right() < bounds.right()).then(|| Position::new(area.right(), center.y)),
        area.y.checked_sub(1).map(|y| Position::new(center.x, y)),
        (area.bottom() < bounds.bottom()).then(|| Position::new(center.x, area.bottom())),
    ];
    let focus_before = harness.focus();
    for pos in candidates.into_iter().flatten() {
        assert!(
            !area.contains(pos),
            "{label}: neighbor probe must stay outside the page"
        );
        let response = harness.click(pos.x, pos.y);
        assert!(
            !response.is_consumed(),
            "{label}: outside click at {},{} was consumed",
            pos.x,
            pos.y
        );
        assert_eq!(
            harness.focus(),
            focus_before,
            "{label}: outside focus changed"
        );
    }
}

fn assert_default_meta_cell<A: TermrockApp>(
    harness: &Harness<A>,
    x: u16,
    y: u16,
    symbol: &str,
    meta_fg: Color,
    canvas: Color,
    label: &str,
) {
    let cell = harness.cell(x, y);
    assert_eq!(cell.symbol(), symbol, "{label}: metadata glyph");
    assert_eq!(cell.fg, meta_fg, "{label}: semantic LIST/META fg");
    assert_eq!(cell.bg, canvas, "{label}: active canvas surface");
    assert_eq!(cell.underline_color, Color::Reset, "{label}: underline");
    assert!(
        !cell.modifier.contains(Modifier::BOLD),
        "{label}: metadata weight"
    );
}

fn assert_resolved_meta_cell<A: TermrockApp>(
    harness: &Harness<A>,
    x: u16,
    y: u16,
    theme: &Theme,
    canvas: Color,
    label: &str,
) {
    let expected = theme.resolve(
        Family::LIST,
        Variant::DEFAULT,
        Part::META,
        StateFlags::empty(),
        Surface::Canvas,
    );
    let cell = harness.cell(x, y);
    assert_eq!(cell.symbol(), "…", "{label}: metadata glyph");
    assert_eq!(cell.fg, expected.style.fg.unwrap(), "{label}: semantic fg");
    assert_eq!(
        cell.bg,
        expected.style.bg.unwrap_or(canvas),
        "{label}: Reset inherits active surface"
    );
    assert_eq!(
        cell.underline_color,
        expected.style.underline_color.unwrap(),
        "{label}: semantic underline"
    );
    assert!(
        cell.modifier.contains(Modifier::ITALIC),
        "{label}: italic override"
    );
    assert!(
        !cell.modifier.contains(Modifier::BOLD),
        "{label}: not title weight"
    );
}

#[test]
fn progress_compact_page_preserves_tiny_metadata_admission() {
    for (color, label, _, meta_fg, canvas) in LEVELS {
        for (origin_x, origin_y) in [(0, 0), (7, 3)] {
            for width in [11, 12, 13] {
                let area = Rect::new(origin_x, origin_y, width, 16);
                let mut harness =
                    direct_page_harness(ProgressPage::new(), Theme::junie(), color, area);
                let title = harness.cell(origin_x, origin_y);
                assert_eq!(title.symbol(), "P", "progress {width} {label}: title");
                assert_eq!(
                    harness.cell(origin_x + 8, origin_y).symbol(),
                    " ",
                    "progress {width} {label}: title/meta gap"
                );
                let meta_x = origin_x + 10;
                match width {
                    11 => assert_eq!(harness.cell(meta_x, origin_y).symbol(), " "),
                    12 => assert_default_meta_cell(
                        &harness,
                        meta_x,
                        origin_y,
                        "…",
                        meta_fg,
                        canvas,
                        &format!("progress width12 {label}"),
                    ),
                    13 => {
                        assert_default_meta_cell(
                            &harness,
                            meta_x,
                            origin_y,
                            "D",
                            meta_fg,
                            canvas,
                            &format!("progress width13 {label}"),
                        );
                        assert_default_meta_cell(
                            &harness,
                            meta_x + 1,
                            origin_y,
                            "…",
                            meta_fg,
                            canvas,
                            &format!("progress width13 ellipsis {label}"),
                        );
                    }
                    _ => unreachable!(),
                }
                assert_eq!(
                    harness.cell(area.right() - 1, origin_y).symbol(),
                    " ",
                    "progress {width} {label}: right edge stays blank"
                );
                assert_eq!(
                    harness.cell(origin_x + 2, origin_y + 5).symbol(),
                    "L",
                    "progress {width} {label}: compact body remains at its old row"
                );
                assert_area_boundaries(
                    &mut harness,
                    area,
                    &format!("progress {width} {label} at {origin_x},{origin_y}"),
                );
            }
        }
    }
}

#[test]
fn settings_compact_page_preserves_tiny_metadata_admission() {
    for (color, label, _, meta_fg, canvas) in LEVELS {
        for (origin_x, origin_y) in [(0, 0), (7, 3)] {
            for width in [19, 20, 21] {
                let area = Rect::new(origin_x, origin_y, width, 16);
                let mut harness =
                    direct_page_harness(SettingsPage::new(), Theme::junie(), color, area);
                let title = harness.cell(origin_x, origin_y);
                assert_eq!(title.symbol(), "P", "settings {width} {label}: title");
                assert_eq!(
                    harness.cell(origin_x + 16, origin_y).symbol(),
                    " ",
                    "settings {width} {label}: title/meta gap"
                );
                let meta_x = origin_x + 18;
                match width {
                    19 => assert_eq!(harness.cell(meta_x, origin_y).symbol(), " "),
                    20 => assert_default_meta_cell(
                        &harness,
                        meta_x,
                        origin_y,
                        "…",
                        meta_fg,
                        canvas,
                        &format!("settings width20 {label}"),
                    ),
                    21 => {
                        assert_default_meta_cell(
                            &harness,
                            meta_x,
                            origin_y,
                            "C",
                            meta_fg,
                            canvas,
                            &format!("settings width21 {label}"),
                        );
                        assert_default_meta_cell(
                            &harness,
                            meta_x + 1,
                            origin_y,
                            "…",
                            meta_fg,
                            canvas,
                            &format!("settings width21 ellipsis {label}"),
                        );
                    }
                    _ => unreachable!(),
                }
                assert_eq!(
                    harness.cell(area.right() - 1, origin_y).symbol(),
                    " ",
                    "settings {width} {label}: right edge stays blank"
                );
                assert_eq!(
                    harness.cell(origin_x + 1, origin_y + 2).symbol(),
                    "P",
                    "settings {width} {label}: tabs body remains at its old row"
                );
                assert!(
                    harness.text().contains("Playground"),
                    "settings {width} {label}: section tabs remain visible"
                );
                assert_area_boundaries(
                    &mut harness,
                    area,
                    &format!("settings {width} {label} at {origin_x},{origin_y}"),
                );
            }
        }
    }
}

#[test]
fn compact_boundary_metadata_keeps_custom_semantic_and_reset_channels() {
    for (color, label, _, _, canvas) in LEVELS {
        let theme = custom_heading_theme(color);
        for (origin_x, origin_y) in [(0, 0), (7, 3)] {
            let progress_area = Rect::new(origin_x, origin_y, 12, 16);
            let mut progress =
                direct_page_harness(ProgressPage::new(), theme.clone(), color, progress_area);
            assert_resolved_meta_cell(
                &progress,
                origin_x + 10,
                origin_y,
                &theme,
                canvas,
                &format!("progress custom {label}"),
            );
            assert_area_boundaries(
                &mut progress,
                progress_area,
                &format!("progress custom {label} at {origin_x},{origin_y}"),
            );

            let settings_area = Rect::new(origin_x, origin_y, 20, 16);
            let mut settings =
                direct_page_harness(SettingsPage::new(), theme.clone(), color, settings_area);
            assert_resolved_meta_cell(
                &settings,
                origin_x + 18,
                origin_y,
                &theme,
                canvas,
                &format!("settings custom {label}"),
            );
            assert_area_boundaries(
                &mut settings,
                settings_area,
                &format!("settings custom {label} at {origin_x},{origin_y}"),
            );
        }
    }
}

fn assert_short_page<P: DemoPage>(
    make_page: impl Fn() -> P,
    label: &str,
    width: u16,
    meta_x: u16,
    title: &str,
) {
    let area = Rect::new(7, 3, width, 1);
    let harness = direct_page_harness(make_page(), Theme::junie(), ColorLevel::TrueColor, area);
    let cell = harness.cell(area.x + meta_x, area.y);
    assert_eq!(
        cell.symbol(),
        "…",
        "{label} short row keeps clipped metadata"
    );
    assert_eq!(
        harness.cell(area.x + width - 1, area.y).symbol(),
        " ",
        "{label} short row keeps its trailing blank"
    );
    assert_eq!(harness.cell(area.x, area.y).symbol(), &title[..1]);
    assert_outside_sentinels(&harness, area, &format!("{label} short row"));

    let empty = Rect::new(7, 3, 0, 16);
    let empty_harness =
        direct_page_harness(make_page(), Theme::junie(), ColorLevel::TrueColor, empty);
    assert_outside_sentinels(&empty_harness, empty, &format!("{label} empty rect"));
}

fn assert_empty_page<P: DemoPage>(make_page: impl Fn() -> P, label: &str) {
    let empty = Rect::new(7, 3, 0, 16);
    let harness = direct_page_harness(make_page(), Theme::junie(), ColorLevel::TrueColor, empty);
    assert_outside_sentinels(&harness, empty, &format!("{label} empty rect"));
}

#[test]
fn compact_metadata_boundaries_handle_short_and_empty_page_rectangles() {
    assert_short_page(ProgressPage::new, "progress", 12, 10, "Progress");
    assert_empty_page(SettingsPage::new, "settings");
}
