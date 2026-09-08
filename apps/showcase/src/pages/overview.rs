//! Overview screen: the public-facade contract and its stable sample data.

use junie_tui::author::PaintStyle;
use junie_tui::{
    Brand, Chord, DerivedHintBar, Empty, EmptyState, Family, FgStep, HelpOverlay, HelpOverlayState,
    HelpSection, Hint, HintBar, HintLayer, Id, ItemKey, KeyCode, KeyHint, Panel, PanelKind, Part,
    Props, PropsList, PropsRow, PropsState, Rect, Role, StateFlags, StylePatch, Surface, TooSmall,
    Ui, Variant, id, layout, width, wrap,
};

use super::{Page, PageUpdate, author::AuthorBadge, frame};

const BRAND: Id = id!("overview.brand");
const AUTHOR: Id = id!("overview.author");
const EMPTY: Id = id!("overview.empty");
const KEY_HINT: Id = id!("overview.key-hint");
const HINT_BAR: Id = id!("overview.hint-bar");
const DERIVED_HINT_BAR: Id = id!("overview.derived-hint-bar");
const HELP: Id = id!("overview.help");
const TOO_SMALL: Id = id!("overview.too-small");
const PROPS_LIST: Id = id!("overview.props-list");
const TOKENS: Id = id!("overview.tokens");
const PRINCIPLES: Id = id!("overview.principles");
const STATE_LANGUAGE: Id = id!("overview.state-language");
const OVERVIEW_PANEL_PARTS: &[(Part, StylePatch)] = &[(
    Part::TITLE,
    StylePatch::new()
        .set_fg(Role::Fg(FgStep::Secondary))
        .remove(junie_tui::Modifier::BOLD),
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

const TOKEN_LABELS: [(&str, &str); 19] = [
    ("canvas", concat!("#", "000000")),
    ("surface", concat!("#", "111111")),
    ("surface.elevated", concat!("#", "18181b")),
    ("surface.overlay", concat!("#", "27272a")),
    ("field", concat!("#", "1e1e22")),
    ("popover", concat!("#", "3f3f46")),
    ("border.subtle", "white 15%"),
    ("border.strong", "white 30%"),
    ("text.primary", concat!("#", "ffffff")),
    ("text.secondary", "white 70%"),
    ("text.muted", "white 50%"),
    ("text.faint", "white 30%"),
    ("accent", concat!("#", "48e054")),
    ("accent.hover", concat!("#", "3ab343")),
    ("accent.pressed", concat!("#", "2b8632")),
    ("accent.bg", "green 20%"),
    ("error", concat!("#", "e44545")),
    ("warning", concat!("#", "f59e09")),
    ("info", concat!("#", "8787ff")),
];

const PRINCIPLE_COPY: [(&str, &str); 5] = [
    (
        "One hue",
        "Green means focus, primary action or selection. Everything else is achromatic.",
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

const STATE_LEGEND: [(&str, &str); 7] = [
    ("▎", "focus"),
    ("░", "hover lifts the surface"),
    ("›", "current / chosen"),
    ("✓", "checked"),
    ("!", "error"),
    ("▁", "editing: cursor + underline"),
    ("○", "disabled: faint, no hover"),
];

fn brand() -> Brand<'static> {
    Brand::new(BRAND, "Junie").tagline("component showcase")
}

fn inventory_hints() -> HintLayer {
    HintLayer {
        hints: vec![Hint {
            key: junie_tui::HintKey::Chord(Chord::key(KeyCode::Char('i'))),
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

/// The landing page has no mutable controls; its content is deliberately
/// useful as a smoke test for themes, clipping and public component exports.
#[derive(Debug)]
pub(crate) struct OverviewPage {
    author: AuthorBadge,
    help_state: HelpOverlayState,
    props_state: PropsState,
}

impl OverviewPage {
    pub(crate) fn new() -> Self {
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

    fn update(&mut self, cx: &mut junie_tui::Cx<'_>) -> PageUpdate {
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
        let _ = brand();
        let _ = props_list();
        let _ = props().draw(ui, Rect::ZERO);
        frame(
            ui,
            area,
            self.title(),
            "Tokens and principles behind every component",
            |ui, body| {
                // Restore the historical responsive split: below 68 columns
                // the token card and the legend stack vertically. This is
                // what preserves the compact 80-column composition.
                let (left, right) = if body.width < 68 {
                    let rows = layout::rows(
                        body,
                        &[
                            junie_tui::Track::Fixed(body.height / 2),
                            junie_tui::Track::Flex(1),
                        ],
                    );
                    (
                        rows.first().copied().unwrap_or(body),
                        rows.get(1).copied().unwrap_or(Rect::ZERO),
                    )
                } else {
                    let columns = layout::columns(
                        body,
                        &[junie_tui::Track::Fixed(46), junie_tui::Track::Flex(1)],
                        2,
                    );
                    (
                        columns.first().copied().unwrap_or(body),
                        columns.get(1).copied().unwrap_or(Rect::ZERO),
                    )
                };
                draw_tokens(ui, left);
                draw_principles(ui, right, &self.author);
            },
        );
    }
}

fn overview_style(ui: &mut Ui<'_>, part: Part, flags: StateFlags) -> PaintStyle {
    ui.style(Family::PANEL, Variant::DEFAULT, part, flags)
        .style
        .with_bg_from(ui.surface_style())
}

fn draw_tokens(ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let token_count = u16::try_from(TOKEN_LABELS.len()).unwrap_or(u16::MAX);
    let area = Rect {
        // The historical compact frame gives the token card only six data
        // rows. Keeping the card short is what lets the second token column
        // and the state legend occupy their legacy positions at 80 columns.
        height: if area.width >= 40 && area.height <= 18 {
            area.height.min(9)
        } else {
            area.height.min(token_count.saturating_add(3))
        },
        ..area
    };
    tokens_panel().draw(ui, area, |ui, inner| {
        let primary = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Primary))));
        let muted = ui
            .surface_style()
            .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
        let colors = [
            Role::Surface(Surface::Canvas),
            Role::Surface(Surface::Surface),
            Role::Surface(Surface::Elevated),
            Role::Surface(Surface::Overlay),
            Role::Surface(Surface::Field),
            Role::Surface(Surface::Popover),
            Role::BorderSubtle,
            Role::BorderStrong,
            Role::Fg(FgStep::Primary),
            Role::Fg(FgStep::Secondary),
            Role::Fg(FgStep::Muted),
            Role::Fg(FgStep::Faint),
            Role::Accent,
            Role::AccentHover,
            Role::AccentPressed,
            Role::AccentTint,
            Role::Danger,
            Role::Warning,
            Role::Info,
        ];
        let two_columns = inner.height < token_count && inner.width >= 40;
        let per_column = if two_columns {
            TOKEN_LABELS.len().div_ceil(2)
        } else {
            TOKEN_LABELS.len()
        };
        let column_width = if two_columns {
            inner.width / 2
        } else {
            inner.width
        };
        for (column, (labels, color_column)) in TOKEN_LABELS
            .chunks(per_column)
            .zip(colors.chunks(per_column))
            .enumerate()
        {
            let Ok(column) = u16::try_from(column) else {
                break;
            };
            for (row, ((name, note), color)) in labels.iter().zip(color_column).enumerate() {
                let Ok(row) = u16::try_from(row) else {
                    break;
                };
                let Some(y) = inner.y.checked_add(row) else {
                    break;
                };
                if y >= inner.bottom() {
                    continue;
                }
                let x = inner.x.saturating_add(column.saturating_mul(column_width));
                let mut swatch = ui.surface_style();
                swatch = swatch.patch(ui.paint_patch(&StylePatch::new().set_bg(*color)));
                ui.fill(Rect::new(x, y, 4, 1), swatch);
                ui.paint_str(Rect::new(x.saturating_add(4), y, 1, 1), "▏", muted);
                ui.paint_str(
                    Rect::new(x.saturating_add(6), y, column_width.saturating_sub(6), 1),
                    name,
                    primary,
                );
                if column_width > 30 {
                    let note_width = width(note);
                    ui.paint_str(
                        Rect::new(
                            x.saturating_add(
                                column_width.saturating_sub(note_width.saturating_add(1)),
                            ),
                            y,
                            note_width,
                            1,
                        ),
                        note,
                        muted,
                    );
                }
            }
        }
    });
}

fn draw_principles(ui: &mut Ui<'_>, area: Rect, author: &AuthorBadge) {
    if area.is_empty() {
        return;
    }
    if area.height <= 10 {
        // At the compact terminal width the principles card is intentionally
        // clipped out of the historical composition. The state card starts
        // below the token rows and owns the visible right-hand legend.
        let state_area = Rect {
            y: area.y.saturating_add(1),
            height: area.height.saturating_sub(1),
            ..area
        };
        if !state_area.is_empty() {
            draw_state_language(ui, state_area);
        }
        return;
    }
    let inner_width = area.width.saturating_sub(4);
    let wrapped: Vec<(&str, Vec<String>)> = PRINCIPLE_COPY
        .iter()
        .map(|(title, text)| (*title, wrap(text, inner_width)))
        .collect();
    let needed = wrapped
        .iter()
        .map(|(_, lines)| {
            u16::try_from(lines.len())
                .unwrap_or(u16::MAX)
                .saturating_add(2)
        })
        .sum::<u16>()
        .saturating_add(2);
    let principles_height = needed.min(area.height.saturating_sub(10));
    let principles_area = Rect {
        height: principles_height,
        ..area
    };
    principles_panel().draw(ui, principles_area, |ui, inner| {
        let title_style = overview_style(ui, Part::TITLE, StateFlags::empty());
        let detail_style = overview_style(ui, Part::DETAIL, StateFlags::empty());
        let mut y = inner.y;
        for (title, lines) in &wrapped {
            if y.saturating_add(1) >= inner.bottom() {
                break;
            }
            ui.paint_str(Rect::new(inner.x, y, inner.width, 1), title, title_style);
            y = y.saturating_add(1);
            for line in lines {
                if y >= inner.bottom() {
                    break;
                }
                ui.paint_str(Rect::new(inner.x, y, inner.width, 1), line, detail_style);
                y = y.saturating_add(1);
            }
            y = y.saturating_add(1);
        }
    });

    let legend_y = principles_area.bottom().saturating_add(1);
    let legend_area = Rect::new(
        area.x,
        legend_y,
        area.width,
        area.bottom().saturating_sub(legend_y),
    );
    if legend_area.is_empty() {
        return;
    }
    let author_y = legend_area.bottom().saturating_sub(1);
    let state_area = Rect {
        height: legend_area.height.saturating_sub(1),
        ..legend_area
    };
    draw_state_language(ui, state_area);
    let author_area = Rect::new(
        area.x,
        author_y,
        area.width,
        area.bottom().saturating_sub(author_y).min(1),
    );
    author.draw(ui, author_area);
}

fn draw_state_language(ui: &mut Ui<'_>, state_area: Rect) {
    state_language_panel().draw(ui, state_area, |ui, inner| {
        for (index, (glyph, label)) in STATE_LEGEND.iter().enumerate() {
            let Ok(offset) = u16::try_from(index) else {
                break;
            };
            let Some(y) = inner.y.checked_add(offset) else {
                break;
            };
            if y >= inner.bottom() {
                break;
            }
            let marker_color = match index {
                0 | 2 | 3 => Role::Accent,
                1 => Role::Fg(FgStep::Secondary),
                4 => Role::Danger,
                5 => Role::Fg(FgStep::Primary),
                _ => Role::Fg(FgStep::Faint),
            };
            ui.paint_str(
                Rect::new(inner.x, y, 1, 1),
                glyph,
                ui.surface_style()
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(marker_color))),
            );
            ui.paint_str(
                Rect::new(
                    inner.x.saturating_add(3),
                    y,
                    inner.width.saturating_sub(3),
                    1,
                ),
                label,
                ui.surface_style()
                    .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)))),
            );
        }
    });
}
