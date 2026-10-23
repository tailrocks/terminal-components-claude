use ratatui_core::layout::Rect;
use ratatui_core::style::Color;
use termrock_feedback::{
    AccountRole, AccountRow, AccountsDetail, DetailAlert, DetailHint, DetailInk, DetailLine,
    DetailSpan, DetailTail, paint_accounts_detail,
};
use termrock_test_support::Scene;
use termrock_theme::{ColorLevel, Theme};

const LEVELS: [ColorLevel; 4] = [
    ColorLevel::TrueColor,
    ColorLevel::Ansi256,
    ColorLevel::Ansi16,
    ColorLevel::Mono,
];
const CLOSED_FULL_CELL_BASELINE: [[u64; 4]; 3] = [
    [
        0xcf11_5aca_3ba8_ecbc,
        0xfa1b_5db6_a398_9501,
        0xc1a5_58c0_0e11_515d,
        0xdc1f_7c95_68fa_2edd,
    ],
    [
        0xd9b2_d2ad_1253_6f58,
        0x5998_9fbe_7a0f_79fc,
        0x5897_35e2_763f_4586,
        0x3dc0_c24b_c2d9_d5e3,
    ],
    [
        0x50d8_22f2_91a1_d5d5,
        0xe226_29c0_6298_2332,
        0x2db7_c7c1_817a_e803,
        0x281c_837f_d721_d76f,
    ],
];

const MENUS: [&str; 2] = ["Home", "Accounts"];
const ROWS: [AccountRow<'static>; 2] = [
    AccountRow {
        role: AccountRole::Group,
        mark: Some("◆"),
        name: "Personal",
        alert: None,
        alert_tone: DetailAlert::Danger,
        tail: Some("2 accounts"),
        tail_tone: DetailTail::Muted,
        selected: true,
        faded: false,
    },
    AccountRow {
        role: AccountRole::Item,
        mark: Some("●"),
        name: "Work",
        alert: Some("!"),
        alert_tone: DetailAlert::Warning,
        tail: Some("Stale"),
        tail_tone: DetailTail::Warning,
        selected: false,
        faded: false,
    },
];
const FACT_SPANS: [DetailSpan<'static>; 4] = [
    DetailSpan {
        text: "login.example",
        ink: DetailInk::Plain,
    },
    DetailSpan {
        text: " · stale",
        ink: DetailInk::Muted,
    },
    DetailSpan {
        text: " · warning",
        ink: DetailInk::Warning,
    },
    DetailSpan {
        text: " · invalid",
        ink: DetailInk::DangerBold,
    },
];
const FACTS: [DetailLine<'static>; 1] = [DetailLine { spans: &FACT_SPANS }];
const HINTS: [DetailHint<'static>; 2] = [
    DetailHint {
        key: "↑↓",
        action: "Move",
    },
    DetailHint {
        key: "Enter",
        action: "Inspect",
    },
];

fn page() -> AccountsDetail<'static> {
    AccountsDetail {
        brand: "Termrock",
        menus: &MENUS,
        crumb: "Accounts / Work",
        status: "READY",
        title: "Accounts",
        summary: "2 items",
        rows: &ROWS,
        total: 2,
        offset: 0,
        heading: "Selected account",
        heading_meta: "WORK",
        facts: &FACTS,
        hints: &HINTS,
        hint_overflow: "+2",
    }
}

fn full_cell_digest(scene: &Scene) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0100_0000_01b3;

    let mut hash = OFFSET;
    for (index, cell) in scene.buffer().content.iter().enumerate() {
        let record = format!("{index}|{cell:?}");
        hash = fnv(hash, record.as_bytes(), PRIME);
        hash = fnv(hash, &[0xff], PRIME);
    }
    hash
}

fn fnv(mut hash: u64, bytes: &[u8], prime: u64) -> u64 {
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(prime);
    }
    hash
}

fn custom_theme() -> Theme {
    let mut tokens = Theme::junie().color;
    tokens.accent = Color::Rgb(0x13, 0x57, 0x9b);
    tokens.on_accent = Color::Rgb(0xf0, 0xe1, 0xd2);
    tokens.border_strong = Color::Rgb(0x4a, 0x8c, 0xb1);
    for (index, color) in tokens.surfaces.iter_mut().enumerate() {
        *color = Color::Rgb(0x10 + index as u8, 0x20 + index as u8, 0x30 + index as u8);
    }
    for (index, color) in tokens.fg.iter_mut().enumerate() {
        *color = Color::Rgb(0xa0 + index as u8, 0xb0 + index as u8, 0xc0 + index as u8);
    }
    Theme::from_tokens(tokens)
}

fn render(theme: Theme, level: ColorLevel) -> Scene {
    let mut scene = Scene::new("accounts-detail-closed", theme, level, 80, 24);
    scene.draw(|ui, area| paint_accounts_detail(ui, area, &page()));
    scene
}

#[test]
fn closed_painter_full_cells_match_prechange_across_themes_and_tiers() {
    for (theme_index, theme) in [Theme::junie(), Theme::paper(), custom_theme()]
        .into_iter()
        .enumerate()
    {
        for (level_index, level) in LEVELS.into_iter().enumerate() {
            let scene = render(theme.clone(), level);
            assert_eq!(
                full_cell_digest(&scene),
                CLOSED_FULL_CELL_BASELINE[theme_index][level_index],
                "theme index {theme_index}, color level {level:?}"
            );
        }
    }
}

#[test]
fn summary_geometry_tracks_the_painted_six_row_band() {
    assert_eq!(
        termrock_feedback::accounts_detail_summary_area(Rect::new(0, 0, 80, 24)),
        Some(Rect::new(1, 16, 78, 6))
    );
    assert_eq!(
        termrock_feedback::accounts_detail_summary_area(Rect::new(7, 5, 40, 16)),
        Some(Rect::new(8, 13, 38, 6))
    );
    assert_eq!(
        termrock_feedback::accounts_detail_summary_area(Rect::new(3, 2, 80, 24)),
        Some(Rect::new(4, 18, 78, 6))
    );
}

#[test]
fn summary_geometry_rejects_unsupported_and_empty_visible_areas() {
    for area in [
        Rect::new(0, 0, 39, 16),
        Rect::new(0, 0, 40, 15),
        Rect::new(0, 0, 0, 0),
        Rect::new(u16::MAX, 0, 40, 16),
        Rect::new(0, u16::MAX, 40, 16),
    ] {
        assert_eq!(
            termrock_feedback::accounts_detail_summary_area(area),
            None,
            "unsupported or invisible area {area:?}"
        );
    }
}
