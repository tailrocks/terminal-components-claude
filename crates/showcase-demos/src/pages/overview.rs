use termrock::author::PaintStyle;
use termrock::{
    Brand, Chord, Color, DerivedHintBar, Empty, EmptyState, FgStep, FieldSpec, Form, HelpOverlay,
    HelpOverlayState, HelpSection, Hint, HintBar, HintLayer, Id, ItemKey, KeyCode, KeyHint, Meter,
    Modifier, Panel, PanelKind, Part, Props, PropsList, PropsRow, PropsState, Rect, Role,
    ScrollRegion, SplitAxis, SplitPane, StylePatch, TooSmall, Ui, Wizard, width, wrap,
};

use super::{Page, PageUpdate, author::AuthorBadge, frame};

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
        frame(ui, area, self.title(), "", |ui, body| {
            paint_blurb(ui, area);
            paint_section_strip(ui, body);
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

const CANVAS: u32 = 0x000000;
const SURFACE: u32 = 0x111111;
const WHITE: u32 = 0xffffff;
const SECONDARY: u32 = 0xb3b3b3;
const MUTED: u32 = 0x8a8a8a;
const FAINT: u32 = 0x737373;
const RULE: u32 = 0x3a3a3a;
const ACCENT: u32 = 0x48e054;
const TAB_ACTIVE: u32 = 0x27272a;
const THUMB: u32 = 0x808080;
const TRACK: u32 = 0x474747;

const PLANES: &[(&str, u32)] = &[
    ("canvas", 0x000000),
    ("surface", 0x111111),
    ("surface.elevated", 0x18181b),
    ("surface.overlay", 0x27272a),
    ("field", 0x1e1e22),
    ("field.hover", 0x2f2f34),
    ("popover", 0x3f3f46),
    ("highlight", 0x2f5aa8),
    ("highlight.danger", 0x7a2a2a),
    ("selection", 0x1f3460),
    ("border.subtle", 0x3a3a3a),
    ("border.strong", 0x4d4d4d),
    ("scroll.track", 0x474747),
    ("text.primary", 0xffffff),
    ("text.secondary", 0xb3b3b3),
    ("text.muted", 0x8a8a8a),
    ("text.faint", 0x737373),
    ("text.ghost", 0x262626),
    ("text.on_accent", 0x19191c),
    ("disabled", 0x4d4d4d),
];

const ACCENTS: &[(&str, u32)] = &[
    ("accent", 0x48e054),
    ("accent.hover", 0x3ab343),
    ("accent.pressed", 0x2e9a38),
    ("accent.bg", 0x173f1c),
    ("accent.bg.subtle", 0x0a1c0c),
    ("accent.bg.hover", 0x1e5024),
    ("focus", 0x48e054),
    ("link", 0x79b8ff),
    ("success", 0x3fb950),
    ("success.strong", 0x56d364),
    ("info", 0x79b8ff),
    ("warning", 0xf59e09),
    ("error", 0xf05d5d),
    ("error.bg", 0x2e0f0f),
    ("error.soft", 0xe8a0a0),
    ("error.strong", 0xff8a8a),
    ("danger", 0xff6b6b),
    ("syntax.keyword", 0x79b8ff),
    ("syntax.string", 0x8bd5ca),
    ("syntax.literal", 0xc6a0f6),
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

fn ink(fg: u32, bg: u32) -> PaintStyle {
    PaintStyle::new().fg(rgb(fg)).bg(rgb(bg))
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
    ui.paint_str(meta, BLURB, ink(MUTED, CANVAS));
}

fn tab_width(label: &str) -> u16 {
    width(label).saturating_add(3)
}

fn paint_section_strip(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 {
            ink(WHITE, TAB_ACTIVE).add_modifier(Modifier::BOLD)
        } else {
            ink(SECONDARY, CANVAS)
        };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= SECTION_WIDE {
        let caption_width = width(SECTION_CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                SECTION_CAPTION,
                ink(FAINT, CANVAS),
            );
        }
    }
    if area.height < 2 {
        return;
    }
    let rule_y = area.y.saturating_add(1);
    ui.paint_str(
        Rect {
            y: rule_y,
            height: 1,
            ..area
        },
        &"─".repeat(usize::from(area.width)),
        ink(RULE, CANVAS),
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        ink(ACCENT, CANVAS),
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

fn paint_playground(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let (view, bar) = scroll_split(area);
    paint_visible_cards(ui, view);
    if let Some(bar) = bar {
        paint_scrollbar(ui, bar, playground_height(view.width));
    }
}

fn scroll_split(area: Rect) -> (Rect, Option<Rect>) {
    let overflows = area.width > SCROLL_ROOM && playground_height(area.width) > area.height;
    if !overflows {
        return (area, None);
    }
    let view = Rect {
        width: area.width.saturating_sub(SCROLL_ROOM),
        ..area
    };
    let bar = Rect::new(area.right().saturating_sub(1), area.y, 1, view.height);
    (view, Some(bar))
}

fn paint_visible_cards(ui: &mut Ui<'_>, view: Rect) {
    let (cols, card_w, groups) = bands(view.width);
    let mut y = view.y;
    for group in groups {
        let band_h = group
            .iter()
            .map(|card| card_height(*card, card_w))
            .max()
            .unwrap_or(0);
        if y >= view.bottom() {
            break;
        }
        for (index, card) in group.iter().enumerate() {
            let Ok(index) = u16::try_from(index) else {
                break;
            };
            let x = view
                .x
                .saturating_add(index.saturating_mul(card_w.saturating_add(CARD_GAP)));
            let w = if index + 1 == cols {
                view.right().saturating_sub(x)
            } else {
                card_w
            };
            paint_card(
                ui,
                Rect::new(x, y, w, card_height(*card, card_w)),
                view,
                *card,
            );
        }
        y = y.saturating_add(band_h).saturating_add(CARD_GAP);
    }
}

fn paint_card(ui: &mut Ui<'_>, card: Rect, view: Rect, kind: OverviewCard) {
    let visible = card.intersection(view);
    if visible.is_empty() {
        return;
    }
    ui.fill(visible, ink(WHITE, SURFACE));
    if card.y >= view.y && card.y < view.bottom() {
        ui.paint_str(
            Rect::new(
                card.x.saturating_add(CARD_PAD),
                card.y,
                card.width.saturating_sub(CARD_PAD),
                1,
            ),
            card_title(kind),
            ink(SECONDARY, SURFACE),
        );
    }
    let rule_y = card.y.saturating_add(1);
    let rule_w = card.width.saturating_sub(CARD_PAD.saturating_mul(2));
    if rule_w > 0 && rule_y >= view.y && rule_y < view.bottom() && rule_y < card.bottom() {
        ui.paint_str(
            Rect::new(card.x.saturating_add(CARD_PAD), rule_y, rule_w, 1),
            &"─".repeat(usize::from(rule_w)),
            ink(RULE, SURFACE),
        );
    }
    let rows: &[(&str, u32)] = match kind {
        OverviewCard::Planes => PLANES,
        OverviewCard::Accents => ACCENTS,
        _ => return,
    };
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
        ui.fill(Rect::new(inner_x, y, swatch, 1), ink(WHITE, *color));
        if inner_w > 4 {
            ui.paint_str(
                Rect::new(inner_x.saturating_add(4), y, 1, 1),
                "▏",
                ink(FAINT, SURFACE),
            );
        }
        if inner_w > 6 {
            ui.paint_str(
                Rect::new(inner_x.saturating_add(6), y, inner_w.saturating_sub(6), 1),
                name,
                ink(WHITE, SURFACE),
            );
        }
        let note = format!("#{color:06x}");
        let note_w = width(&note);
        let note_at = inner_w.saturating_sub(note_w);
        if note_at > width(name).saturating_add(6) {
            ui.paint_str(
                Rect::new(inner_x.saturating_add(note_at), y, note_w, 1),
                &note,
                ink(MUTED, SURFACE),
            );
        }
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
                ink(THUMB, CANVAS)
            } else {
                ink(TRACK, CANVAS)
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
}
