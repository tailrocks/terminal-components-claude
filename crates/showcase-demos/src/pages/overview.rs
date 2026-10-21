use termrock::author::PaintStyle;
use termrock::{
    Brand, Chord, Color, ColorLevel, DerivedHintBar, Empty, EmptyState, FgStep, FieldSpec, Form,
    HelpOverlay, HelpOverlayState, HelpSection, Hint, HintBar, HintLayer, Id, ItemKey, KeyCode,
    KeyHint, Meter, Panel, PanelKind, Part, Props, PropsList, PropsRow, PropsState, Rect,
    Role, ScrollRegion, SplitAxis, SplitPane, StylePatch, TooSmall, Ui, Wizard, width, wrap,
};

use super::{Page, PageUpdate, author::AuthorBadge, chrome_ink};
use termrock::feedback::{PageFrame, SectionStrip};

const BRAND: Id = termrock::Id::root("showcase_ui::pages::overview::overview.brand");
const AUTHOR: Id = termrock::Id::root("showcase_ui::pages::overview::overview.author");
const EMPTY: Id = termrock::Id::root("showcase_ui::pages::overview::overview.empty");
const KEY_HINT: Id = termrock::Id::root("showcase_ui::pages::overview::overview.key-hint");
const HINT_BAR: Id = termrock::Id::root("showcase_ui::pages::overview::overview.hint-bar");
const DERIVED_HINT_BAR: Id =
    termrock::Id::root("showcase_ui::pages::overview::overview.derived-hint-bar");
const HELP: Id = termrock::Id::root("showcase_ui::pages::overview::overview.help");
const TOO_SMALL: Id = termrock::Id::root("showcase_ui::pages::overview::overview.too-small");
const PROPS_LIST: Id = termrock::Id::root("showcase_ui::pages::overview::overview.props-list");
const TOKENS: Id = termrock::Id::root("showcase_ui::pages::overview::overview.tokens");
const PRINCIPLES: Id = termrock::Id::root("showcase_ui::pages::overview::overview.principles");
const STATE_LANGUAGE: Id =
    termrock::Id::root("showcase_ui::pages::overview::overview.state-language");
const COMPONENT_ROSTER: Id =
    termrock::Id::root("showcase_ui::pages::overview::overview.component-roster");
const OVERVIEW_PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(termrock::Modifier::BOLD),
)];
const PROPS_LIST_ROWS: [PropsRow<'static>; 3] = [
    PropsRow::new(ItemKey::num(1), "Library", "junie-tui"),
    PropsRow::new(ItemKey::num(2), "Ownership", "application state"),
    PropsRow::new(ItemKey::num(3), "Rendering", "public Ui facade"),
];
const PROPS_ROWS: [(&str, &str); 3] = [
    ("Library", "junie-tui"),
    ("Ownership", "application state"),
    ("Rendering", "public Ui facade"),
];

fn component_roster() {
    let _ = Form::new(COMPONENT_ROSTER, &[] as &[FieldSpec<'static>]);
    let _ = Meter::new(COMPONENT_ROSTER);
    let _ = ScrollRegion::new(COMPONENT_ROSTER);
    let _ = SplitPane::new(COMPONENT_ROSTER, SplitAxis::Horizontal);
    let _ = Wizard::new(COMPONENT_ROSTER, &[]);
}

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "Junie").tagline("component showcase")
}

fn inventory_hints() -> HintLayer {
    HintLayer {
        hints: vec![Hint {
            key: termrock::HintKey::Chord(Chord::key(KeyCode::Char('i'))),
            label: "inspect",
            priority: 50,
        }],
        badge: Some("API"),
        status: None,
        centered: false,
    }
}

fn derived_hint_bar() -> DerivedHintBar<'static> {
    let derived: DerivedHintBar<'static> = HintBar::derived(DERIVED_HINT_BAR);
    derived
}

fn props_list() -> PropsList<'static> {
    PropsList::new(PROPS_LIST)
}

fn props() -> Props<'static> {
    Props::new(&PROPS_ROWS)
}

fn tokens_panel() -> Panel<'static> {
    Panel::new(TOKENS)
        .kind(PanelKind::Card)
        .title("Tokens")
        .patch_part(OVERVIEW_PANEL_PARTS)
}

fn principles_panel() -> Panel<'static> {
    Panel::new(PRINCIPLES)
        .kind(PanelKind::Card)
        .title("Principles")
        .patch_part(OVERVIEW_PANEL_PARTS)
}

fn state_language_panel() -> Panel<'static> {
    Panel::new(STATE_LANGUAGE)
        .kind(PanelKind::Card)
        .title("State language")
        .patch_part(OVERVIEW_PANEL_PARTS)
}

#[derive(Debug)]
pub struct OverviewPage {
    author: AuthorBadge,
    help_state: HelpOverlayState,
    props_state: PropsState,
}

impl OverviewPage {
    pub fn new() -> Self {
        Self {
            author: AuthorBadge::new(AUTHOR),
            help_state: HelpOverlayState::default(),
            props_state: PropsState::default(),
        }
    }
}

impl Default for OverviewPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for OverviewPage {
    fn title(&self) -> &'static str {
        "Overview"
    }

    fn update(&mut self, cx: &mut termrock::Cx<'_>) -> PageUpdate {
        let mut response = brand().update(cx).erase();
        response |= self.author.update(cx);
        let hints = inventory_hints();
        let sections = [HelpSection::new("Overview", &hints)];
        response |= HelpOverlay::new(HELP, "overview", &sections)
            .update(cx, &mut self.help_state)
            .erase();
        response |= props_list()
            .update(cx, &mut self.props_state, &PROPS_LIST_ROWS)
            .erase();
        let _ = props();
        let _ = tokens_panel();
        let _ = principles_panel();
        let _ = state_language_panel();
        let _ = Empty::new(
            EMPTY,
            EmptyState::Empty {
                title: "No optional content",
                hint: Some("the app owns this empty state"),
            },
        );
        let _ = KeyHint::new(KEY_HINT, Chord::key(KeyCode::Char('i')), "inspect");
        let _ = HintBar::new(HINT_BAR, &hints);
        let _ = derived_hint_bar();
        let _ = TooSmall::new(TOO_SMALL, "showcase");
        response.into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        component_roster();
        let _ = brand();
        let _ = props_list();
        let _ = props().draw(ui, Rect::ZERO);
        PageFrame::new(self.title())
            .meta("")
            .draw_with(ui, area, |ui, body| {
                paint_blurb(ui, area);
                SectionStrip::new(&SECTIONS)
                    .caption(SECTION_CAPTION)
                    .caption_min_width(SECTION_WIDE)
                    .ink(chrome_ink())
                    .draw(ui, body);
                let content = Rect {
                    y: body.y.saturating_add(3),
                    height: body.height.saturating_sub(3),
                    ..body
                };
                paint_playground(ui, content);
            });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[("[ ]", "Pages"), ("i", "Inspector")]
    }
}

const BLURB: &str = "Tokens and principles behind every component";
const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
const SECTION_CAPTION: &str = "the live widget";
const SECTION_WIDE: u16 = 76;
const CARD_MIN: u16 = 50;
const CARD_GAP: u16 = 1;
const CARD_PAD: u16 = 2;
const SCROLL_ROOM: u16 = 2;
// Settings sheet outer width: label column 22, value 34, card pad 4.
const SETTINGS_MIN: u16 = 60;

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;

fn planes_and_text(t: &termrock::Theme) -> [(&'static str, Color); 20] {
    match t.capability.color {
        ColorLevel::Ansi256 => [
            ("canvas", Color::Indexed(16)),
            ("surface", Color::Indexed(232)),
            ("surface.elevated", Color::Indexed(233)),
            ("surface.overlay", Color::Indexed(235)),
            ("field", Color::Indexed(234)),
            ("field.hover", Color::Indexed(237)),
            ("popover", Color::Indexed(237)),
            ("highlight", Color::Indexed(25)),
            ("highlight.danger", Color::Indexed(88)),
            ("selection", Color::Indexed(17)),
            ("border.subtle", Color::Indexed(238)),
            ("border.strong", Color::Indexed(242)),
            ("scroll.track", Color::Indexed(239)),
            ("text.primary", Color::Indexed(231)),
            ("text.secondary", Color::Indexed(249)),
            ("text.muted", Color::Indexed(245)),
            ("text.faint", Color::Indexed(242)),
            ("text.ghost", Color::Indexed(238)),
            ("text.on_accent", Color::Indexed(232)),
            ("disabled", Color::Indexed(239)),
        ],
        ColorLevel::Ansi16 => [
            ("canvas", Color::Black),
            ("surface", Color::Black),
            ("surface.elevated", Color::Black),
            ("surface.overlay", Color::DarkGray),
            ("field", Color::Black),
            ("field.hover", Color::DarkGray),
            ("popover", Color::DarkGray),
            ("highlight", Color::Blue),
            ("highlight.danger", Color::Red),
            ("selection", Color::Black),
            ("border.subtle", Color::DarkGray),
            ("border.strong", Color::Gray),
            ("scroll.track", Color::DarkGray),
            ("text.primary", Color::White),
            ("text.secondary", Color::Gray),
            ("text.muted", Color::Gray),
            ("text.faint", Color::Gray),
            ("text.ghost", Color::DarkGray),
            ("text.on_accent", Color::Black),
            ("disabled", Color::DarkGray),
        ],
        ColorLevel::Mono => [
            ("canvas", Color::Black),
            ("surface", Color::Black),
            ("surface.elevated", Color::Black),
            ("surface.overlay", Color::DarkGray),
            ("field", Color::Black),
            ("field.hover", Color::Black),
            ("popover", Color::DarkGray),
            ("highlight", Color::DarkGray),
            ("highlight.danger", Color::DarkGray),
            ("selection", Color::DarkGray),
            ("border.subtle", Color::DarkGray),
            ("border.strong", Color::Gray),
            ("scroll.track", Color::DarkGray),
            ("text.primary", Color::White),
            ("text.secondary", Color::Gray),
            ("text.muted", Color::Gray),
            ("text.faint", Color::Gray),
            ("text.ghost", Color::DarkGray),
            ("text.on_accent", Color::Black),
            ("disabled", Color::DarkGray),
        ],
        ColorLevel::TrueColor | _ => [
            ("canvas", rgb(0x000000)),
            ("surface", rgb(0x111111)),
            ("surface.elevated", rgb(0x18181b)),
            ("surface.overlay", rgb(0x27272a)),
            ("field", rgb(0x1e1e22)),
            ("field.hover", rgb(0x2f2f34)),
            ("popover", rgb(0x3f3f46)),
            ("highlight", rgb(0x2f5aa8)),
            ("highlight.danger", rgb(0x7a2a2a)),
            ("selection", rgb(0x1f3460)),
            ("border.subtle", rgb(0x3a3a3a)),
            ("border.strong", rgb(0x4d4d4d)),
            ("scroll.track", rgb(0x474747)),
            ("text.primary", rgb(0xffffff)),
            ("text.secondary", rgb(0xb3b3b3)),
            ("text.muted", rgb(0x8a8a8a)),
            ("text.faint", rgb(0x737373)),
            ("text.ghost", rgb(0x262626)),
            ("text.on_accent", rgb(0x19191c)),
            ("disabled", rgb(0x4d4d4d)),
        ],
    }
}

fn accent_and_status(t: &termrock::Theme) -> [(&'static str, Color); 20] {
    match t.capability.color {
        ColorLevel::Ansi256 => [
            ("accent", Color::Indexed(78)),
            ("accent.hover", Color::Indexed(71)),
            ("accent.pressed", Color::Indexed(65)),
            ("accent.bg", Color::Indexed(22)),
            ("accent.bg.subtle", Color::Indexed(22)),
            ("accent.bg.hover", Color::Indexed(28)),
            ("focus", Color::Indexed(78)),
            ("link", Color::Indexed(111)),
            ("success", Color::Indexed(71)),
            ("success.strong", Color::Indexed(77)),
            ("info", Color::Indexed(111)),
            ("warning", Color::Indexed(214)),
            ("error", Color::Indexed(203)),
            ("error.bg", Color::Indexed(52)),
            ("error.soft", Color::Indexed(174)),
            ("error.strong", Color::Indexed(210)),
            ("danger", Color::Indexed(203)),
            ("syntax.keyword", Color::Indexed(111)),
            ("syntax.string", Color::Indexed(116)),
            ("syntax.literal", Color::Indexed(183)),
        ],
        ColorLevel::Ansi16 => [
            ("accent", Color::LightGreen),
            ("accent.hover", Color::Green),
            ("accent.pressed", Color::Green),
            ("accent.bg", Color::Black),
            ("accent.bg.subtle", Color::Black),
            ("accent.bg.hover", Color::DarkGray),
            ("focus", Color::LightGreen),
            ("link", Color::LightBlue),
            ("success", Color::Green),
            ("success.strong", Color::Green),
            ("info", Color::LightBlue),
            ("warning", Color::Yellow),
            ("error", Color::LightRed),
            ("error.bg", Color::Black),
            ("error.soft", Color::LightRed),
            ("error.strong", Color::LightRed),
            ("danger", Color::LightRed),
            ("syntax.keyword", Color::LightBlue),
            ("syntax.string", Color::Cyan),
            ("syntax.literal", Color::LightMagenta),
        ],
        ColorLevel::Mono => [
            ("accent", Color::White),
            ("accent.hover", Color::White),
            ("accent.pressed", Color::White),
            ("accent.bg", Color::Black),
            ("accent.bg.subtle", Color::Black),
            ("accent.bg.hover", Color::Black),
            ("focus", Color::White),
            ("link", Color::Gray),
            ("success", Color::White),
            ("success.strong", Color::White),
            ("info", Color::Gray),
            ("warning", Color::Gray),
            ("error", Color::White),
            ("error.bg", Color::Black),
            ("error.soft", Color::White),
            ("error.strong", Color::White),
            ("danger", Color::White),
            ("syntax.keyword", Color::Gray),
            ("syntax.string", Color::Gray),
            ("syntax.literal", Color::White),
        ],
        ColorLevel::TrueColor | _ => [
            ("accent", rgb(0x48e054)),
            ("accent.hover", rgb(0x3ab343)),
            ("accent.pressed", rgb(0x2e9a38)),
            ("accent.bg", rgb(0x173f1c)),
            ("accent.bg.subtle", rgb(0x0a1c0c)),
            ("accent.bg.hover", rgb(0x1e5024)),
            ("focus", rgb(0x48e054)),
            ("link", rgb(0x79b8ff)),
            ("success", rgb(0x3fb950)),
            ("success.strong", rgb(0x56d364)),
            ("info", rgb(0x79b8ff)),
            ("warning", rgb(0xf59e09)),
            ("error", rgb(0xf05d5d)),
            ("error.bg", rgb(0x2e0f0f)),
            ("error.soft", rgb(0xe8a0a0)),
            ("error.strong", rgb(0xff8a8a)),
            ("danger", rgb(0xff6b6b)),
            ("syntax.keyword", rgb(0x79b8ff)),
            ("syntax.string", rgb(0x8bd5ca)),
            ("syntax.literal", rgb(0xc6a0f6)),
        ],
    }
}

const GLYPH_SAMPLES: &[(&str, &str, &str)] = &[
    ("checks", "[✓] [ ] [–]", "[x] [ ] [-]"),
    ("radio, toggle", "(●) ( ) ──● ○──", "(*) ( ) --* o--"),
    ("chevrons", "▴ ▾ ▾ ▸", "^ v v >"),
    ("status", "✓ ✗ ▲ i", "+ X ! i"),
    ("scrollbar", "┆┃┆", ":#:"),
    ("meter", "━━─", "==-"),
    ("more, close", "… ▾ ×", "~ v x"),
];

const STRING_SAMPLES: &[(&str, &str, &str)] = &[
    ("more_cut", "… {} more", "... {} more"),
    ("search", "Type to search…", "Type to search..."),
    ("empty_list", "Nothing here yet", "Nothing here yet"),
    ("loading", "Loading rows…", "Loading rows..."),
];

const LADDER_PLANES: &[(&str, [u32; 3], &str)] = &[
    ("Canvas", [0x000000, 0x1c1c1f, 0x303036], "Raised"),
    ("Surface", [0x111111, 0x26262a, 0x38383e], "Raised"),
    ("Card", [0x18181b, 0x2a2a2f, 0x3c3c43], "Raised"),
    ("Field", [0x1e1e22, 0x2f2f34, 0x3a3a40], "Raised"),
    ("Raised", [0x27272a, 0x37373d, 0x4a4a52], "Popover"),
    ("Popover", [0x3f3f46, 0x4f4f58, 0x63636d], "Popover"),
    ("Highlight", [0x2f5aa8, 0x3a6bc4, 0x244a8c], "Highlight"),
    (
        "HighlightDanger",
        [0x7a2a2a, 0x8c3030, 0x5a1f1f],
        "HighlightDanger",
    ),
    ("Accent", [0x48e054, 0x3ab343, 0x2e9a38], "Accent"),
    ("Danger", [0xd93636, 0xc22e2e, 0x9e2424], "Danger"),
];

const LADDER_INKS: &[(&str, [u32; 3], &str)] = &[
    ("handle", [0x808080, 0xb3b3b3, 0x48e054], "┃"),
    ("seam", [0x4d4d4d, 0x808080, 0x48e054], "│"),
];

const PRINCIPLE_COPY: &[(&str, &str)] = &[
    (
        "Colour carries meaning",
        "One accent hue for focus and primary action; status hues only where they carry meaning; syntax hues only in code.",
    ),
    (
        "Never colour alone",
        "Every status has its glyph: ✓ done, ✗ failed, ▲ attention, • pending, i info.",
    ),
    (
        "Alpha ladder",
        "Text and borders step down in white opacity, never in arbitrary grays.",
    ),
    (
        "State is geometry",
        "Hover lifts the surface, focus adds a bar, selection adds a marker, editing shows the cursor.",
    ),
    (
        "Three planes",
        "Canvas, surface, elevated. Depth comes from lightness, not borders.",
    ),
    (
        "Quiet chrome",
        "Bold is reserved for the focused control. No box around a thing unless the box carries meaning.",
    ),
];

#[derive(Clone, Copy)]
enum OverviewCard {
    Planes,
    Accents,
    Glyphs,
    Ladders,
    Contrast,
    Principles,
    Roles,
    Explorer,
    Mods,
    Settings,
    Legend,
    Spacing,
}

const OVERVIEW_CARDS: [OverviewCard; 12] = [
    OverviewCard::Planes,
    OverviewCard::Accents,
    OverviewCard::Glyphs,
    OverviewCard::Ladders,
    OverviewCard::Contrast,
    OverviewCard::Principles,
    OverviewCard::Roles,
    OverviewCard::Explorer,
    OverviewCard::Mods,
    OverviewCard::Settings,
    OverviewCard::Legend,
    OverviewCard::Spacing,
];

fn rgb(hex: u32) -> Color {
    Color::Rgb(
        ((hex >> 16) & 0xff) as u8,
        ((hex >> 8) & 0xff) as u8,
        (hex & 0xff) as u8,
    )
}

fn token_color(t: &termrock::Theme, hex: u32) -> Color {
    match t.capability.color {
        ColorLevel::Ansi256 => match hex {
            CANVAS => Color::Indexed(16),
            SURFACE => Color::Indexed(232),
            WHITE => Color::Indexed(231),
            SECONDARY => Color::Indexed(249),
            MUTED => Color::Indexed(245),
            FAINT => Color::Indexed(242),
            RULE => Color::Indexed(238),
            ACCENT => Color::Indexed(78),
            TAB_ACTIVE => Color::Indexed(235),
            THUMB => Color::Indexed(245),
            TRACK => Color::Indexed(239),
            _ => rgb(hex),
        },
        ColorLevel::Ansi16 => match hex {
            CANVAS | SURFACE => Color::Black,
            WHITE => Color::White,
            SECONDARY | MUTED | FAINT | THUMB => Color::Gray,
            RULE | TAB_ACTIVE | TRACK => Color::DarkGray,
            ACCENT => Color::LightGreen,
            _ => rgb(hex),
        },
        ColorLevel::Mono => match hex {
            CANVAS | SURFACE => Color::Black,
            WHITE | ACCENT => Color::White,
            SECONDARY | MUTED | FAINT | THUMB => Color::Gray,
            RULE | TAB_ACTIVE | TRACK => Color::DarkGray,
            _ => rgb(hex),
        },
        ColorLevel::TrueColor | _ => rgb(hex),
    }
}

fn ink(ui: &Ui<'_>, fg: u32, bg: u32) -> PaintStyle {
    let t = ui.theme_ref();
    PaintStyle::new()
        .fg(token_color(t, fg))
        .bg(token_color(t, bg))
}

fn paint_blurb(ui: &mut Ui<'_>, area: Rect) {
    let title_width = width("Overview");
    if area.width <= title_width.saturating_add(4) {
        return;
    }
    let meta = Rect {
        x: area.x.saturating_add(title_width).saturating_add(2),
        width: area.width.saturating_sub(title_width).saturating_sub(3),
        height: 1,
        ..area
    };
    let style = ink(ui, MUTED, CANVAS);
    if width(BLURB) <= meta.width {
        ui.paint_str(meta, BLURB, style);
        return;
    }
    let budget = Rect {
        width: meta.width.saturating_sub(1),
        ..meta
    };
    let used = ui.paint_str(budget, BLURB, style);
    ui.paint_str(
        Rect::new(meta.x.saturating_add(used), meta.y, 1, 1),
        "…",
        style,
    );
}

fn card_title(card: OverviewCard) -> &'static str {
    match card {
        OverviewCard::Planes => "Tokens · planes and text",
        OverviewCard::Accents => "Tokens · accent and status",
        OverviewCard::Glyphs => "Glyphs and strings",
        OverviewCard::Ladders => "Ladders · theme.ladders",
        OverviewCard::Contrast => "Contrast on the card, per level",
        OverviewCard::Principles => "Principles",
        OverviewCard::Roles => "Roles",
        OverviewCard::Explorer => "Explorer",
        OverviewCard::Mods => "State modifiers · theme.mods",
        OverviewCard::Settings => "Theme settings",
        OverviewCard::Legend => "State language",
        OverviewCard::Spacing => "Spacing · buttons · badges",
    }
}

fn card_height(card: OverviewCard, card_w: u16) -> u16 {
    let body = match card {
        OverviewCard::Planes | OverviewCard::Accents => 20,
        OverviewCard::Glyphs => 14,
        OverviewCard::Ladders => 13,
        OverviewCard::Contrast => 12,
        OverviewCard::Principles => {
            principles_rows(card_w.saturating_sub(CARD_PAD.saturating_mul(2)))
        }
        OverviewCard::Roles => return 12,
        OverviewCard::Explorer => 10,
        OverviewCard::Mods => 10,
        OverviewCard::Settings => 8,
        OverviewCard::Legend => 7,
        OverviewCard::Spacing => 9,
    };
    body.saturating_add(3)
}

fn principles_rows(inner: u16) -> u16 {
    PRINCIPLE_COPY
        .iter()
        .map(|(title, text)| {
            let line = format!("{title}. {text}");
            u16::try_from(wrap(&line, inner.max(1)).len()).unwrap_or(u16::MAX)
        })
        .fold(0u16, u16::saturating_add)
}

fn bands(width: u16) -> (u16, u16, Vec<Vec<OverviewCard>>) {
    let cols = (width.saturating_add(1) / (CARD_MIN + 1)).clamp(1, 3);
    let card_w =
        width.saturating_sub(CARD_GAP.saturating_mul(cols.saturating_sub(1))) / cols.max(1);
    let groups = OVERVIEW_CARDS
        .chunks(usize::from(cols))
        .map(<[OverviewCard]>::to_vec)
        .collect();
    (cols, card_w, groups)
}

fn playground_height(width: u16) -> u16 {
    if width == 0 {
        return 0;
    }
    let (_, card_w, groups) = bands(width);
    let mut total = 0u16;
    for (index, group) in groups.iter().enumerate() {
        let height = group
            .iter()
            .map(|card| card_height(*card, card_w))
            .max()
            .unwrap_or(0);
        total = total.saturating_add(height);
        if index + 1 != groups.len() {
            total = total.saturating_add(CARD_GAP);
        }
    }
    total
}

struct ScrollSplit {
    clip: Rect,
    content_w: u16,
    content_h: u16,
    vertical: Option<Rect>,
    horizontal: Option<Rect>,
}

fn content_width(avail: u16) -> u16 {
    avail.max(CARD_MIN).max(SETTINGS_MIN)
}

fn scroll_split(area: Rect) -> ScrollSplit {
    let mut width = content_width(area.width);
    let mut height = playground_height(width);
    if height > area.height && area.width > SCROLL_ROOM && width <= area.width {
        let narrow = area.width.saturating_sub(SCROLL_ROOM);
        let narrowed = content_width(narrow);
        if narrowed <= narrow {
            width = narrowed;
            height = playground_height(width);
        }
    }
    let mut vertical = false;
    let mut horizontal = false;
    for _ in 0..3 {
        let view_w = area
            .width
            .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
        let view_h = area.height.saturating_sub(u16::from(horizontal));
        let next_v = height > view_h;
        let next_h = width > view_w;
        if (next_v, next_h) == (vertical, horizontal) {
            break;
        }
        vertical = next_v;
        horizontal = next_h;
    }
    vertical = vertical && area.width > SCROLL_ROOM;
    horizontal = horizontal && area.height > 1;
    let view_w = area
        .width
        .saturating_sub(if vertical { SCROLL_ROOM } else { 0 });
    let view_h = area.height.saturating_sub(u16::from(horizontal));
    ScrollSplit {
        clip: Rect::new(area.x, area.y, view_w, view_h),
        content_w: width.max(view_w),
        content_h: height,
        vertical: vertical.then(|| Rect::new(area.right().saturating_sub(1), area.y, 1, view_h)),
        horizontal: horizontal.then(|| {
            Rect::new(
                area.x,
                area.bottom().saturating_sub(1),
                area.width.saturating_sub(u16::from(vertical)),
                1,
            )
        }),
    }
}

fn paint_playground(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let split = scroll_split(area);
    paint_visible_cards(ui, split.clip, split.content_w);
    if let Some(bar) = split.vertical {
        paint_scrollbar(ui, bar, split.content_h);
    }
    if let Some(bar) = split.horizontal {
        paint_hscrollbar(ui, bar, split.clip.width, split.content_w);
    }
}

fn paint_visible_cards(ui: &mut Ui<'_>, clip: Rect, content_w: u16) {
    let (cols, card_w, groups) = bands(content_w);
    let mut y = clip.y;
    let content_right = clip.x.saturating_add(content_w);
    for group in groups {
        let band_h = group
            .iter()
            .map(|card| card_height(*card, card_w))
            .max()
            .unwrap_or(0);
        if y >= clip.bottom() {
            break;
        }
        for (index, card) in group.iter().enumerate() {
            let Ok(index) = u16::try_from(index) else {
                break;
            };
            let x = clip
                .x
                .saturating_add(index.saturating_mul(card_w.saturating_add(CARD_GAP)));
            let w = if index + 1 == cols {
                content_right.saturating_sub(x)
            } else {
                card_w
            };
            paint_card(
                ui,
                Rect::new(x, y, w, card_height(*card, card_w)),
                clip,
                *card,
            );
        }
        y = y.saturating_add(band_h).saturating_add(CARD_GAP);
    }
}

fn card_theme_colors(t: &termrock::Theme) -> (Color, Color, Color, Color, Color, Color) {
    match t.capability.color {
        ColorLevel::Ansi256 => (
            Color::Indexed(232),
            Color::Indexed(231),
            Color::Indexed(249),
            Color::Indexed(245),
            Color::Indexed(242),
            Color::Indexed(238),
        ),
        ColorLevel::Ansi16 => (
            Color::Black,
            Color::White,
            Color::Gray,
            Color::Gray,
            Color::DarkGray,
            Color::DarkGray,
        ),
        ColorLevel::Mono => (
            Color::Black,
            Color::White,
            Color::Gray,
            Color::Gray,
            Color::DarkGray,
            Color::DarkGray,
        ),
        ColorLevel::TrueColor | _ => (
            rgb(SURFACE),
            rgb(WHITE),
            rgb(SECONDARY),
            rgb(MUTED),
            rgb(FAINT),
            rgb(RULE),
        ),
    }
}

fn paint_card(ui: &mut Ui<'_>, card: Rect, view: Rect, kind: OverviewCard) {
    let visible = card.intersection(view);
    if visible.is_empty() {
        return;
    }
    let t = ui.theme_ref();
    let (surface, primary, secondary, _muted, _faint, rule) = card_theme_colors(t);
    ui.fill(visible, PaintStyle::new().fg(primary).bg(surface));
    if card.y >= view.y && card.y < view.bottom() {
        paint_in(
            ui,
            view,
            Rect::new(
                card.x.saturating_add(CARD_PAD),
                card.y,
                card.width.saturating_sub(CARD_PAD),
                1,
            ),
            card_title(kind),
            PaintStyle::new().fg(secondary).bg(surface),
        );
    }
    let rule_y = card.y.saturating_add(1);
    let rule_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    if rule_w > 0 && rule_y >= view.y && rule_y < view.bottom() && rule_y < card.bottom() {
        paint_in(
            ui,
            view,
            Rect::new(card.x.saturating_add(CARD_PAD), rule_y, rule_w, 1),
            &"─".repeat(usize::from(rule_w)),
            PaintStyle::new().fg(rule).bg(surface),
        );
    }
    match kind {
        OverviewCard::Planes => paint_swatches(ui, view, card, &planes_and_text(t)),
        OverviewCard::Accents => paint_swatches(ui, view, card, &accent_and_status(t)),
        OverviewCard::Glyphs => paint_glyphs(ui, view, card),
        OverviewCard::Ladders => paint_ladders(ui, view, card),
        _ => {}
    }
}

fn swatch_note(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(i) => format!("256: {i}"),
        other => format!("{other:?}"),
    }
}

fn paint_swatches(ui: &mut Ui<'_>, view: Rect, card: Rect, rows: &[(&str, Color)]) {
    let (surface, primary, _secondary, muted, faint, _rule) = card_theme_colors(ui.theme_ref());
    let inner_x = card.x.saturating_add(CARD_PAD);
    let inner_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    for (index, (name, color)) in rows.iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let y = card.y.saturating_add(2).saturating_add(index);
        if y >= view.bottom() || y >= card.bottom() {
            break;
        }
        if y < view.y || inner_w == 0 {
            continue;
        }
        let swatch = 4.min(inner_w);
        let swatch_rect = Rect::new(inner_x, y, swatch, 1).intersection(view);
        if !swatch_rect.is_empty() {
            ui.fill(swatch_rect, PaintStyle::new().fg(primary).bg(*color));
        }
        if inner_w > 4 {
            paint_in(
                ui,
                view,
                Rect::new(inner_x.saturating_add(4), y, 1, 1),
                "▏",
                PaintStyle::new().fg(faint).bg(surface),
            );
        }
        if inner_w > 6 {
            paint_in(
                ui,
                view,
                Rect::new(inner_x.saturating_add(6), y, inner_w.saturating_sub(6), 1),
                name,
                PaintStyle::new().fg(primary).bg(surface),
            );
        }
        let note = swatch_note(*color);
        let note_w = width(&note);
        let note_at = inner_w.saturating_sub(note_w);
        if note_at > width(name).saturating_add(6) {
            paint_in(
                ui,
                view,
                Rect::new(inner_x.saturating_add(note_at), y, note_w, 1),
                &note,
                PaintStyle::new().fg(muted).bg(surface),
            );
        }
    }
}

fn card_body(card: Rect) -> Rect {
    Rect::new(
        card.x.saturating_add(CARD_PAD),
        card.y.saturating_add(2),
        card.width.saturating_sub(CARD_PAD.saturating_mul(2)),
        card.height.saturating_sub(2),
    )
}

fn paint_body(
    ui: &mut Ui<'_>,
    view: Rect,
    body: Rect,
    row: u16,
    col: u16,
    text: &str,
    style: PaintStyle,
) {
    if row >= body.height || col >= body.width || text.is_empty() {
        return;
    }
    paint_in(
        ui,
        view,
        Rect::new(
            body.x.saturating_add(col),
            body.y.saturating_add(row),
            body.width.saturating_sub(col),
            1,
        ),
        text,
        style,
    );
}

fn paint_glyphs(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    let body = card_body(card);
    if body.is_empty() {
        return;
    }
    let muted = ink(ui, MUTED, SURFACE);
    let name = ink(ui, WHITE, SURFACE);
    let value = ink(ui, SECONDARY, SURFACE);
    paint_body(ui, view, body, 0, 0, "Glyphs", muted);
    paint_body(ui, view, body, 0, 15, "UNICODE", muted);
    paint_body(ui, view, body, 0, 33, "ASCII", muted);
    for (index, (label, unicode, ascii)) in GLYPH_SAMPLES.iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let row = index.saturating_add(1);
        paint_body(ui, view, body, row, 0, label, name);
        paint_body(ui, view, body, row, 15, unicode, value);
        paint_body(ui, view, body, row, 33, ascii, value);
    }
    paint_body(ui, view, body, 9, 0, "Strings", muted);
    paint_body(ui, view, body, 9, 15, "DEFAULT", muted);
    paint_body(ui, view, body, 9, 33, "override", muted);
    for (index, (label, unicode, ascii)) in STRING_SAMPLES.iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let row = index.saturating_add(10);
        paint_body(ui, view, body, row, 0, label, name);
        paint_body(ui, view, body, row, 15, unicode, value);
        paint_body(ui, view, body, row, 33, ascii, value);
    }
}

fn paint_ladders(ui: &mut Ui<'_>, view: Rect, card: Rect) {
    let body = card_body(card);
    if body.is_empty() {
        return;
    }
    let muted = ink(ui, MUTED, SURFACE);
    let name = ink(ui, WHITE, SURFACE);
    paint_body(
        ui,
        view,
        body,
        0,
        0,
        "Ladder: rest hover pressed · Plane::up",
        muted,
    );
    for (index, (label, colors, up)) in LADDER_PLANES.iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let row = index.saturating_add(1);
        paint_body(ui, view, body, row, 0, label, name);
        for (step, color) in colors.iter().enumerate() {
            let Ok(step) = u16::try_from(step) else {
                break;
            };
            let x = 16u16.saturating_add(step.saturating_mul(5));
            if row >= body.height || x.saturating_add(4) > body.width {
                continue;
            }
            let swatch = Rect::new(body.x.saturating_add(x), body.y.saturating_add(row), 4, 1)
                .intersection(view);
            if !swatch.is_empty() {
                ui.fill(swatch, ink(ui, WHITE, *color));
            }
        }
        paint_body(ui, view, body, row, 32, &format!("→ {up}"), muted);
    }
    for (index, (label, colors, glyph)) in LADDER_INKS.iter().enumerate() {
        let Ok(index) = u16::try_from(index) else {
            break;
        };
        let row = 11u16.saturating_add(index);
        paint_body(ui, view, body, row, 0, label, name);
        for (step, color) in colors.iter().enumerate() {
            let Ok(step) = u16::try_from(step) else {
                break;
            };
            paint_body(
                ui,
                view,
                body,
                row,
                17u16.saturating_add(step.saturating_mul(5)),
                glyph,
                ink(ui, *color, SURFACE),
            );
        }
    }
}

fn paint_in(ui: &mut Ui<'_>, view: Rect, rect: Rect, text: &str, style: PaintStyle) {
    let rect = rect.intersection(view);
    if rect.is_empty() || text.is_empty() {
        return;
    }
    ui.paint_str(rect, text, style);
}

fn paint_hscrollbar(ui: &mut Ui<'_>, bar: Rect, viewport: u16, content_w: u16) {
    let track_len = usize::from(bar.width);
    let viewport = usize::from(viewport);
    let content = usize::from(content_w);
    let thumb = if content <= viewport || track_len == 0 {
        track_len
    } else {
        viewport
            .saturating_mul(track_len)
            .checked_div(content)
            .unwrap_or(0)
            .max(1)
            .min(track_len)
    };
    for col in 0..bar.width {
        let on_thumb = usize::from(col) < thumb;
        ui.paint_str(
            Rect::new(bar.x.saturating_add(col), bar.y, 1, 1),
            if on_thumb { "━" } else { "┄" },
            if on_thumb {
                ink(ui, THUMB, CANVAS)
            } else {
                ink(ui, TRACK, CANVAS)
            },
        );
    }
}

fn paint_scrollbar(ui: &mut Ui<'_>, bar: Rect, content_h: u16) {
    let track_len = usize::from(bar.height);
    let viewport = track_len;
    let content = usize::from(content_h);
    let thumb = if content <= viewport || track_len == 0 {
        track_len
    } else {
        viewport
            .saturating_mul(track_len)
            .checked_div(content)
            .unwrap_or(0)
            .max(1)
            .min(track_len)
    };
    for row in 0..bar.height {
        let on_thumb = usize::from(row) < thumb;
        ui.paint_str(
            Rect::new(bar.x, bar.y.saturating_add(row), 1, 1),
            if on_thumb { "┃" } else { "┆" },
            if on_thumb {
                ink(ui, THUMB, CANVAS)
            } else {
                ink(ui, TRACK, CANVAS)
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{OverviewPage, Page};
    use termrock::{App, Color, Cx, Modifier, Response, Theme, Ui};
    use termrock_test_support::Harness;

    struct Host;

    impl App for Host {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            OverviewPage::new().draw(ui, ui.full());
        }
    }

    #[test]
    fn playground_matches_the_tag_section_strip_and_token_cards() {
        let h = Harness::new(Host, Theme::junie(), 94, 36);
        let text = h.text();
        assert!(text.contains("Playground"), "{text}");
        assert!(text.contains(" Options"));
        assert!(text.contains(" States"));
        assert!(text.contains("the live widget"));
        assert!(text.contains("Tokens · planes and text"));
        assert!(text.contains("Tokens · accent and status"));
        assert!(text.contains("accent.bg.subtle"));
        assert!(text.contains("#0a1c0c"));
        let (x, y) = h.find("Playground").expect("playground tab");
        let tab = h.cell(x, y);
        assert_eq!(tab.bg, Color::Rgb(0x27, 0x27, 0x2a));
        assert!(tab.modifier.contains(Modifier::BOLD));
        assert_eq!(h.cell(x, y + 1).symbol(), "━");
        let (sx, sy) = h.find("canvas").expect("canvas swatch");
        assert_eq!(h.cell(sx.saturating_sub(6), sy).bg, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn narrow_overview_blurb_ends_on_the_ellipsis() {
        let h = Harness::new(Host, Theme::junie(), 51, 6);
        assert_eq!(h.cell(48, 0).symbol(), "p");
        assert_eq!(h.cell(49, 0).symbol(), "…");
        assert_eq!(h.cell(49, 0).fg, Color::Rgb(0x8a, 0x8a, 0x8a));
        assert_eq!(h.cell(49, 0).bg, Color::Rgb(0, 0, 0));
    }

    #[test]
    fn wide_overview_paints_glyph_samples_and_ladder_swatches() {
        let h = Harness::new(Host, Theme::junie(), 132, 50);
        let text = h.text();
        assert!(text.contains("[✓] [ ] [–]"), "{text}");
        assert!(text.contains("Type to search…"), "{text}");
        assert!(
            text.contains("Ladder: rest hover pressed · Plane::up"),
            "{text}"
        );
        assert!(text.contains("→ HighlightDanger"), "{text}");
        let (x, y) = h.find("Canvas").expect("canvas ladder");
        assert_eq!(h.cell(x + 16, y).bg, Color::Rgb(0, 0, 0));
        assert_eq!(h.cell(x + 21, y).bg, Color::Rgb(0x1c, 0x1c, 0x1f));
        assert_eq!(h.cell(x + 26, y).bg, Color::Rgb(0x30, 0x30, 0x36));
        let (hx, hy) = h.find("handle").expect("handle ladder");
        assert_eq!(h.cell(hx + 17, hy).symbol(), "┃");
        assert_eq!(h.cell(hx + 17, hy).fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(h.cell(hx + 27, hy).fg, Color::Rgb(0x48, 0xe0, 0x54));
    }
}
