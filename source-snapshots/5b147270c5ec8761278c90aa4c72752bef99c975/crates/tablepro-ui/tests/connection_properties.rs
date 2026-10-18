//! WI-TABLEPRO-CONNECTION-PROPS: the details card paints through stock
//! `Props::rich` (per-row tone + wrap) instead of the hand-painted
//! `draw_connection_properties`.
//!
//! Hardcoded literals below are captured from base `23d601d2a` control dumps
//! (probe `/tmp/propsprobe` + `/tmp/v2check`, oracle bytes in
//! `baselines/tuiscotti-v1/tablepro/connections/`); no test loads a frame file.
//!
//! * T1 pins the 120x40 Production region cell-for-cell (symbols + fg/bg/mods).
//! * T2 pins the shifted wide sizes (100x30 x37, 160x50 x45).
//! * T3 pins the small sizes (no card via the `< 80` compact switch).
//! * T4 pins the per-state wrap splits + tones over downs 0..6 (Scratch
//!   repeats at downs 1-2: SAME bytes asserted).
//! * T5a pins the Production+Silent warning input at component level (the
//!   form cannot create that fixture at base, so there is no app-level T5).
//! * T6 pins stock ownership: PROPS LABEL/META recipe overrides repaint the
//!   card (legacy `paint_patch` bypasses recipes, so (a)/(b) FAIL on base),
//!   and the ring gains no props stop.
//! * T7a pins the 82x20 bottom-2 clip at component level (12 visual rows in
//!   an 11-row budget cuts "Last used"; the 82x21 boundary fits all 12).
//!
//! T5b/T7b (app-level warning/clip observation) are BLOCKED at base: the
//! connection form is inert (`journey_tablepro_connection_form` times out),
//! so no Harness driver can stage Production+Silent. See plan section 2e.

use std::cell::Cell;

use tablepro_ui::TableProApp;
use termrock::{
    App, Color, Cx, Family, FgStep, ItemKey, KeyCode, Part, Props, PropsRow, Response, Role,
    StylePatch, Theme, Ui, Variant,
};
use termrock_test_support::Harness;

const WHITE: Color = Color::Rgb(255, 255, 255);
const SECOND: Color = Color::Rgb(179, 179, 179);
const MUTED: Color = Color::Rgb(128, 128, 128);
const FAINT: Color = Color::Rgb(77, 77, 77);
const CARD: Color = Color::Rgb(17, 17, 17);
const INFO: Color = Color::Rgb(135, 135, 255);
const WARNING: Color = Color::Rgb(245, 158, 9);

/// Symbols of row `y` over `x0..=x1`, one `String` per cell joined.
fn row_span<A: App>(t: &Harness<A>, y: u16, x0: u16, x1: u16) -> String {
    (x0..=x1).map(|x| t.cell(x, y).symbol()).collect()
}

fn assert_cell<A: App>(t: &Harness<A>, x: u16, y: u16, sym: &str, fg: Color, bg: Color, mods: u16) {
    let cell = t.cell(x, y);
    assert_eq!(cell.symbol(), sym, "({x},{y}) symbol");
    assert_eq!(cell.fg, fg, "({x},{y}) fg");
    assert_eq!(cell.bg, bg, "({x},{y}) bg");
    assert_eq!(cell.modifier.bits(), mods, "({x},{y}) mods");
}

/// One painted visual line of the properties card: optional label, the value
/// line text, and the value tone. `dy` is the row offset from the card inner
/// top (Engine row).
struct CardLine {
    dy: u16,
    label: Option<&'static str>,
    value: &'static str,
    tone: Color,
}

/// Production card lines (120x40/100x30/160x50): 8 single-line rows + a
/// 2-line Safe wrap = 9 visual lines at dy 0..=8.
const PRODUCTION_LINES: &[CardLine] = &[
    CardLine {
        dy: 0,
        label: Some("Engine"),
        value: "PostgreSQL",
        tone: WHITE,
    },
    CardLine {
        dy: 1,
        label: Some("Host"),
        value: "prod-db-1.acme.io:5432",
        tone: WHITE,
    },
    CardLine {
        dy: 2,
        label: Some("Database"),
        value: "acme_prod",
        tone: WHITE,
    },
    CardLine {
        dy: 3,
        label: Some("User"),
        value: "acme_ops",
        tone: WHITE,
    },
    CardLine {
        dy: 4,
        label: Some("Environment"),
        value: "production",
        tone: WHITE,
    },
    CardLine {
        dy: 5,
        label: Some("Safe Mode"),
        value: "Safe Mode \u{b7} Writes ask for confirmation and a",
        tone: WHITE,
    },
    CardLine {
        dy: 6,
        label: None,
        value: "deliberate acknowledgement.",
        tone: WHITE,
    },
    CardLine {
        dy: 7,
        label: Some("SSL / SSH"),
        value: "on / bastion.acme.io",
        tone: SECOND,
    },
    CardLine {
        dy: 8,
        label: Some("Last used"),
        value: "1 hour ago",
        tone: MUTED,
    },
];

/// Assert every cell of the properties region: label cells Muted, value cells
/// in the row tone, all other cells in the pre-state filler (White fg on
/// card bg — both painters paint text only).
fn assert_card_region(t: &Harness<TableProApp>, x0: u16, y0: u16, width: u16, tag: &str) {
    for line in PRODUCTION_LINES {
        let y = y0 + line.dy;
        let label_len = line.label.map_or(0, |l| l.chars().count());
        let value_chars: Vec<char> = line.value.chars().collect();
        for dx in 0..width {
            let x = x0 + dx;
            let di = dx as usize;
            let (sym, fg) = if di < label_len {
                (
                    line.label.unwrap().chars().nth(di).unwrap().to_string(),
                    MUTED,
                )
            } else if di < 13 {
                // Label gutter: text-only paint leaves the pre-state filler
                // (White fg) on every row, labeled or not.
                (" ".to_owned(), WHITE)
            } else if di - 13 < value_chars.len() {
                (value_chars[di - 13].to_string(), line.tone)
            } else {
                // Value-column filler: text-only paint leaves the filler.
                (" ".to_owned(), WHITE)
            };
            assert_cell(t, x, y, &sym, fg, CARD, 0);
        }
    }
    // Blank rows below the painted lines (through y0+12), then the action row.
    let _ = tag;
    for y in y0 + 9..=y0 + 12 {
        for dx in 0..width {
            assert_cell(t, x0 + dx, y, " ", WHITE, CARD, 0);
        }
    }
}

/// T1: 120x40 Production region, cell-for-cell (x44..109 x y4..12 + blanks
/// y13..16 + action-row adjacency at y17).
#[test]
fn connection_properties_production_120() {
    let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    for _ in 0..7 {
        let _ = t.key(KeyCode::Down);
    }
    assert_card_region(&t, 44, 4, 66, "120x40");
    assert_eq!(
        row_span(&t, 17, 44, 84),
        " Connect    Edit    Duplicate    Delete… ",
        "120x40: action row must sit directly below the blank rows"
    );
}

/// T2: shifted wide sizes carry the same Production card (100x30 x37 w60,
/// 160x50 x45 w66).
#[test]
fn connection_properties_production_shifted() {
    for (w, h, x0, width) in [(100u16, 30u16, 37u16, 60u16), (160u16, 50u16, 45u16, 66u16)] {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        for _ in 0..7 {
            let _ = t.key(KeyCode::Down);
        }
        assert_card_region(&t, x0, 4, width, &format!("{w}x{h}"));
        assert_eq!(
            row_span(&t, 17, x0, x0 + 40),
            " Connect    Edit    Duplicate    Delete… ",
            "{w}x{h}: action row adjacency"
        );
    }
}

/// T3: narrow sizes show no details card (compact switch, not the TooSmall
/// gate: both sizes draw the normal compact list).
#[test]
fn connection_properties_absent_small() {
    for (w, h) in [(72u16, 20u16), (80u16, 24u16)] {
        let t = Harness::new(TableProApp::default(), Theme::junie(), w, h);
        assert!(
            t.find("SSL / SSH").is_none(),
            "{w}x{h}: details-only marker must be absent"
        );
        assert!(
            t.find("Last used").is_none(),
            "{w}x{h}: details-only row must be absent"
        );
    }
}

/// Per-state T4 expectations at 120x40 (value col x57): engine/host/db/user
/// values, env value + tone, safe wrapped lines + tone, ssl/last values.
struct StateExpect {
    engine: &'static str,
    host: &'static str,
    database: &'static str,
    user: &'static str,
    env: &'static str,
    env_tone: Color,
    safe: &'static [&'static str],
    safe_tone: Color,
    ssl: &'static str,
    last: &'static str,
}

const STATE_EXPECTS: &[StateExpect] = &[
    // downs=0 Local PostgreSQL
    StateExpect {
        engine: "PostgreSQL",
        host: "localhost:5432",
        database: "acme_dev",
        user: "postgres",
        env: "local",
        env_tone: FAINT,
        safe: &[
            "Silent \u{b7} Writes run without asking. Destructive",
            "statements still confirm.",
        ],
        safe_tone: SECOND,
        ssl: "off / off",
        last: "2 minutes ago",
    },
    // downs=1 Scratch
    StateExpect {
        engine: "SQLite",
        host: "~/scratch.db",
        database: "scratch",
        user: "\u{2014}",
        env: "local",
        env_tone: FAINT,
        safe: &[
            "Silent \u{b7} Writes run without asking. Destructive",
            "statements still confirm.",
        ],
        safe_tone: SECOND,
        ssl: "off / off",
        last: "never",
    },
    // downs=2 Scratch (repeat of downs=1)
    StateExpect {
        engine: "SQLite",
        host: "~/scratch.db",
        database: "scratch",
        user: "\u{2014}",
        env: "local",
        env_tone: FAINT,
        safe: &[
            "Silent \u{b7} Writes run without asking. Destructive",
            "statements still confirm.",
        ],
        safe_tone: SECOND,
        ssl: "off / off",
        last: "never",
    },
    // downs=3 Development
    StateExpect {
        engine: "PostgreSQL",
        host: "dev-db.internal.acme.io:5432",
        database: "acme_dev",
        user: "acme_app",
        env: "development",
        env_tone: MUTED,
        safe: &[
            "Silent \u{b7} Writes run without asking. Destructive",
            "statements still confirm.",
        ],
        safe_tone: SECOND,
        ssl: "on / off",
        last: "yesterday",
    },
    // downs=4 Staging
    StateExpect {
        engine: "PostgreSQL",
        host: "staging-db.acme.io:5432",
        database: "acme_staging",
        user: "acme_app",
        env: "staging",
        env_tone: SECOND,
        safe: &[
            "Alert \u{b7} Every write asks for confirmation before it",
            "runs.",
        ],
        safe_tone: SECOND,
        ssl: "on / bastion.acme.io",
        last: "3 days ago",
    },
    // downs=5 Analytics
    StateExpect {
        engine: "MySQL",
        host: "analytics.acme.io:3306",
        database: "warehouse",
        user: "analyst",
        env: "production",
        env_tone: WHITE,
        safe: &[
            "Read-Only \u{b7} Writes are refused. Reads and exports",
            "still work.",
        ],
        safe_tone: WHITE,
        ssl: "on / off",
        last: "last week",
    },
    // downs=6 Production
    StateExpect {
        engine: "PostgreSQL",
        host: "prod-db-1.acme.io:5432",
        database: "acme_prod",
        user: "acme_ops",
        env: "production",
        env_tone: WHITE,
        safe: &[
            "Safe Mode \u{b7} Writes ask for confirmation and a",
            "deliberate acknowledgement.",
        ],
        safe_tone: WHITE,
        ssl: "on / bastion.acme.io",
        last: "1 hour ago",
    },
];

fn dump_region(t: &Harness<TableProApp>, x0: u16, y0: u16, x1: u16, y1: u16) -> String {
    let mut out = String::new();
    for y in y0..=y1 {
        out.push_str(&row_span(t, y, x0, x1));
        out.push('\n');
    }
    out
}

/// T4: per-fixture wrap splits + tones over downs 0..6 (downs=2 repeats
/// downs=1 byte-for-byte).
#[test]
fn connection_properties_wrap_per_fixture() {
    let mut downs1_dump = String::new();
    for (downs, exp) in STATE_EXPECTS.iter().enumerate() {
        let mut t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
        for _ in 0..downs {
            let _ = t.key(KeyCode::Down);
        }
        let tag = format!("downs={downs}");
        // Single-line rows: symbols + first-value-char tone.
        for (y, value, tone) in [
            (4u16, exp.engine, WHITE),
            (5, exp.host, WHITE),
            (6, exp.database, WHITE),
            (7, exp.user, WHITE),
            (8, exp.env, exp.env_tone),
            (11, exp.ssl, SECOND),
            (12, exp.last, MUTED),
        ] {
            assert_eq!(
                row_span(&t, y, 57, 57 + value.chars().count() as u16 - 1),
                value,
                "{tag}: y{y} value symbols"
            );
            assert_eq!(t.cell(57, y).fg, tone, "{tag}: y{y} value tone");
        }
        // Safe wrap: recorded splits, row tone, unlabeled continuation.
        for (i, line) in exp.safe.iter().enumerate() {
            let y = 9 + i as u16;
            assert_eq!(
                row_span(&t, y, 57, 57 + line.chars().count() as u16 - 1),
                *line,
                "{tag}: safe line {i} symbols"
            );
            assert_eq!(t.cell(57, y).fg, exp.safe_tone, "{tag}: safe line {i} tone");
        }
        assert_eq!(
            row_span(&t, 10, 44, 56),
            " ".repeat(13),
            "{tag}: safe continuation must be unlabeled"
        );
        assert_eq!(
            row_span(&t, 9, 44, 52),
            "Safe Mode",
            "{tag}: safe label on visual line 0"
        );
        if downs == 1 {
            downs1_dump = dump_region(&t, 44, 2, 109, 17);
        }
        if downs == 2 {
            assert_eq!(
                dump_region(&t, 44, 2, 109, 17),
                downs1_dump,
                "downs=2 must repeat downs=1 (Scratch) byte-for-byte"
            );
        }
    }
}

/// The exact 9-row Production+Silent input (8 fixture rows + the `""`-label
/// warning row). Shared by the T5a/T7a component pins.
fn silent_production_rows() -> Vec<(String, String, Role, bool)> {
    let safe = format!(
        "Silent \u{b7} {}",
        "Writes run without asking. Destructive statements still confirm."
    );
    [
        (
            "Engine".to_owned(),
            "PostgreSQL".to_owned(),
            Role::Fg(FgStep::Primary),
            false,
        ),
        (
            "Host".to_owned(),
            "prod-db-1.acme.io:5432".to_owned(),
            Role::Fg(FgStep::Primary),
            false,
        ),
        (
            "Database".to_owned(),
            "acme_prod".to_owned(),
            Role::Fg(FgStep::Primary),
            false,
        ),
        (
            "User".to_owned(),
            "acme_ops".to_owned(),
            Role::Fg(FgStep::Primary),
            false,
        ),
        (
            "Environment".to_owned(),
            "production".to_owned(),
            Role::Fg(FgStep::Primary),
            false,
        ),
        (
            "Safe Mode".to_owned(),
            safe,
            Role::Fg(FgStep::Secondary),
            true,
        ),
        (
            String::new(),
            "Production with Silent safe mode: writes run without asking".to_owned(),
            Role::Warning,
            true,
        ),
        (
            "SSL / SSH".to_owned(),
            "on / bastion.acme.io".to_owned(),
            Role::Fg(FgStep::Secondary),
            false,
        ),
        (
            "Last used".to_owned(),
            "1 hour ago".to_owned(),
            Role::Fg(FgStep::Muted),
            false,
        ),
    ]
    .into_iter()
    .collect()
}

/// Minimal component probe: draws the 9-row input with stock `Props::rich`
/// in a fixed rect and records the returned height.
struct RichProbe {
    area: termrock::Rect,
    rows: Vec<(String, String, Role, bool)>,
    painted: Cell<u16>,
}

impl App for RichProbe {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::consumed()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let full = ui.full();
        ui.fill(full, ui.surface_style());
        let rows: Vec<PropsRow<'_>> = self
            .rows
            .iter()
            .enumerate()
            .map(|(i, (label, value, role, wrap))| {
                PropsRow::new(ItemKey::index(i), label, value)
                    .tone(*role)
                    .wrap_if(*wrap)
            })
            .collect();
        let ret = Props::rich(&rows).draw(ui, self.area);
        self.painted.set(ret.height);
    }
}

/// T5a: the 9-row Production+Silent input wraps Safe [47,25] (Secondary) +
/// warning [52,6] (Warning) at value width 53; every other row single-line.
#[test]
fn connection_properties_warning_component() {
    let area = termrock::Rect {
        x: 0,
        y: 0,
        width: 66,
        height: 13,
    };
    let probe = RichProbe {
        area,
        rows: silent_production_rows(),
        painted: Cell::new(0),
    };
    let t = Harness::new(probe, Theme::junie(), 66, 13);
    // Safe row: 2 lines [47,25], Secondary, unlabeled continuation.
    assert_eq!(
        row_span(&t, 5, 13, 59),
        "Silent \u{b7} Writes run without asking. Destructive",
        "safe line 0 (47 cols)"
    );
    assert_eq!(
        row_span(&t, 6, 13, 37),
        "statements still confirm.",
        "safe line 1 (25 cols)"
    );
    assert_eq!(t.cell(13, 5).fg, SECOND, "safe tone");
    assert_eq!(t.cell(13, 6).fg, SECOND, "safe continuation tone");
    assert_eq!(
        row_span(&t, 6, 0, 12),
        " ".repeat(13),
        "safe continuation unlabeled"
    );
    assert_eq!(row_span(&t, 5, 0, 8), "Safe Mode", "safe label on line 0");
    // Warning row: 2 lines [52,6], Warning tone, unlabeled continuation.
    assert_eq!(
        row_span(&t, 7, 13, 64),
        "Production with Silent safe mode: writes run without",
        "warning line 0 (52 cols)"
    );
    assert_eq!(row_span(&t, 8, 13, 18), "asking", "warning line 1 (6 cols)");
    assert_eq!(t.cell(13, 7).fg, WARNING, "warning tone");
    assert_eq!(t.cell(13, 8).fg, WARNING, "warning continuation tone");
    assert_eq!(
        row_span(&t, 8, 0, 12),
        " ".repeat(13),
        "warning continuation unlabeled"
    );
    // All other rows single-line: each label paints exactly once.
    for label in [
        "Engine",
        "Host",
        "Database",
        "User",
        "Environment",
        "Safe Mode",
        "SSL / SSH",
        "Last used",
    ] {
        assert_eq!(t.count(label), 1, "{label:?} must paint exactly once");
    }
    assert_eq!(
        t.count("Production with Silent safe mode"),
        1,
        "warning line 0 must paint exactly once"
    );
    // Row tones on the single-line values.
    for (y, tone) in [
        (0u16, WHITE),
        (1, WHITE),
        (2, WHITE),
        (3, WHITE),
        (4, WHITE),
        (9, SECOND),
        (10, MUTED),
    ] {
        assert_eq!(t.cell(13, y).fg, tone, "y{y} value tone");
    }
}

/// T6: stock ownership — (a) a PROPS LABEL recipe override repaints value
/// cells only; (b) a PROPS META override repaints labels only; (c) the ring
/// gains no props stop. (a)/(b) FAIL on base (legacy `paint_patch` bypasses
/// family recipes) and PASS on the candidate.
#[test]
fn connection_properties_ownership() {
    // (a) LABEL override repaints the values, not the labels.
    let label_mut = Theme::junie().override_variant(Family::PROPS, Variant::DEFAULT, |r| {
        r.part(Part::LABEL)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut tm = Harness::new(TableProApp::default(), label_mut, 120, 40);
    for _ in 0..7 {
        let _ = tm.key(KeyCode::Down);
    }
    assert_eq!(
        tm.cell(57, 4).fg,
        INFO,
        "LABEL override must repaint the Engine value"
    );
    assert_eq!(
        tm.cell(57, 11).fg,
        INFO,
        "LABEL override must repaint the SSL value"
    );
    assert_eq!(
        tm.cell(44, 4).fg,
        MUTED,
        "LABEL override must not touch the labels"
    );
    // (b) META override repaints the labels, not the values.
    let meta_mut = Theme::junie().override_variant(Family::PROPS, Variant::DEFAULT, |r| {
        r.part(Part::META)
            .base(StylePatch::new().set_fg(Role::Info));
    });
    let mut tl = Harness::new(TableProApp::default(), meta_mut, 120, 40);
    for _ in 0..7 {
        let _ = tl.key(KeyCode::Down);
    }
    assert_eq!(
        tl.cell(44, 4).fg,
        INFO,
        "META override must repaint the labels"
    );
    assert_eq!(
        tl.cell(57, 4).fg,
        WHITE,
        "META override must not touch the values"
    );
    // (c) Props registers nothing: no props stop on the ring.
    let t = Harness::new(TableProApp::default(), Theme::junie(), 120, 40);
    let ring: Vec<String> = t
        .ring()
        .entries()
        .iter()
        .map(|e| format!("{:?}", e.id))
        .collect();
    assert!(
        ring.iter().all(|e| !e.contains("props")),
        "ring must gain no props stop (got {ring:?})"
    );
}

/// T7a: the 82x20 clip at component level — 12 visual rows in an 11-row
/// budget (48-col area shortened exactly as the replacement does) cuts
/// "Last used"; the 82x21 boundary (12 rows) fits all 12.
#[test]
fn connection_properties_clip_component() {
    // Firing case: 48x13 shortened to 11 rows.
    let firing = RichProbe {
        area: termrock::Rect {
            x: 0,
            y: 0,
            width: 48,
            height: 11,
        },
        rows: silent_production_rows(),
        painted: Cell::new(0),
    };
    let t = Harness::new(firing, Theme::junie(), 48, 13);
    assert_eq!(
        row_span(&t, 5, 13, 47),
        "Silent \u{b7} Writes run without asking.",
        "safe line 0 at width 35"
    );
    assert_eq!(
        row_span(&t, 6, 13, 40),
        "Destructive statements still",
        "safe line 1 at width 35"
    );
    assert_eq!(
        row_span(&t, 7, 13, 20),
        "confirm.",
        "safe line 2 at width 35"
    );
    assert_eq!(
        row_span(&t, 8, 13, 45),
        "Production with Silent safe mode:",
        "warning line 0 at width 35"
    );
    assert_eq!(
        row_span(&t, 9, 13, 37),
        "writes run without asking",
        "warning line 1 at width 35"
    );
    assert_eq!(
        row_span(&t, 10, 0, 8),
        "SSL / SSH",
        "SSL is the last painted row"
    );
    assert!(
        t.find("Last used").is_none(),
        "the clip must cut the Last-used row"
    );
    assert_eq!(
        t.app().painted.get(),
        11,
        "draw must report 11 painted rows"
    );
    // Boundary case: 48x14 shortened to 12 rows fits everything.
    let boundary = RichProbe {
        area: termrock::Rect {
            x: 0,
            y: 0,
            width: 48,
            height: 12,
        },
        rows: silent_production_rows(),
        painted: Cell::new(0),
    };
    let b = Harness::new(boundary, Theme::junie(), 48, 14);
    assert_eq!(
        b.find("Last used"),
        Some((0, 11)),
        "the 12-row boundary must paint Last-used"
    );
    assert_eq!(
        b.app().painted.get(),
        12,
        "draw must report 12 painted rows"
    );
}
