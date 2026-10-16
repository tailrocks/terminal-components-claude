use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::pages::{Hint, Page, PageCtx, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{EnvKind, StatusKind, SyntaxTone, Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::widgets::panel::Panel;

/// Tokens and principles. Nothing interactive: this page is the reference.
pub struct OverviewPage;

impl OverviewPage {
    pub fn new() -> Self {
        Self
    }
}

fn swatches(t: &Theme) -> Vec<(&'static str, Color)> {
    vec![
        ("canvas", t.canvas),
        ("surface", t.surface),
        ("surface.elevated", t.surface_elevated),
        ("surface.overlay", t.surface_overlay),
        ("field", t.field),
        ("popover", t.popover),
        ("border.subtle", t.border_subtle),
        ("border.strong", t.border_strong),
        ("text.primary", t.text_primary),
        ("text.secondary", t.text_secondary),
        ("text.muted", t.text_muted),
        ("text.faint", t.text_faint),
        ("accent", t.accent),
        ("accent.hover", t.accent_hover),
        ("accent.pressed", t.accent_pressed),
        ("accent.bg", t.accent_bg),
        ("highlight", t.highlight),
        ("selection", t.selection_bg),
        ("success", t.success),
        ("info", t.info),
        ("warning", t.warning),
        ("error", t.error),
        ("danger", t.danger_label),
    ]
}

/// The swatch's value as the terminal receives it: the hex at truecolor,
/// the palette index at 256, the colour name at 16. Read from the theme,
/// so the label never drifts from the token.
fn swatch_note(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02x}{g:02x}{b:02x}"),
        Color::Indexed(i) => format!("256: {i}"),
        other => format!("{other:?}"),
    }
}

const PRINCIPLES: &[(&str, &str)] = &[
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

impl Page for OverviewPage {
    fn title(&self) -> &'static str {
        "Overview"
    }
    fn blurb(&self) -> &'static str {
        "Tokens and principles behind every component"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let (left, right) = crate::pages::layout::columns(area, 46, 2);

        // token swatches
        let panel = Panel::card(Some("Tokens"));
        let bg = panel.bg(t);
        let all = swatches(t);
        let column = left;
        let left = Rect::new(
            left.x,
            left.y,
            left.width,
            left.height.min(all.len() as u16 + 3),
        );
        let inner = panel.render(left, buf, t);
        // two columns when the card cannot show every token in one
        let two_col = (inner.height as usize) < all.len() && inner.width >= 44;
        let per_col = if two_col {
            all.len().div_ceil(2)
        } else {
            all.len()
        };
        let col_w = if two_col {
            inner.width / 2
        } else {
            inner.width
        };
        for (i, (name, color)) in all.iter().enumerate() {
            let note = swatch_note(*color);
            let col = (i / per_col) as u16;
            let y = inner.y + (i % per_col) as u16;
            if y >= inner.bottom() {
                continue;
            }
            let x = inner.x + col * col_w;
            fill(
                buf,
                Rect::new(x, y, 4, 1),
                ratatui::style::Style::new().bg(*color),
            );
            // a faint edge keeps surface-coloured swatches visible on the card
            buf.set_string(x + 4, y, "▏", t.faint().bg(bg));
            buf.set_string(x + 6, y, name, t.primary().bg(bg));
            let nw = junie_tui::ui::text::width(&note) as u16;
            if col_w > 30 {
                buf.set_string(x + col_w.saturating_sub(nw + 1), y, &note, t.muted().bg(bg));
            }
        }

        let below = Rect::new(
            column.x,
            left.bottom() + 1,
            column.width,
            column.bottom().saturating_sub(left.bottom() + 1),
        );
        render_roles(below, buf, t);

        // principles + state legend
        let inner_w = right.width.saturating_sub(4) as usize;
        let wrapped: Vec<(&str, Vec<String>)> = PRINCIPLES
            .iter()
            .map(|(title, text)| (*title, junie_tui::ui::text::wrap(text, inner_w)))
            .collect();
        let needed: u16 = wrapped.iter().map(|(_, l)| l.len() as u16 + 2).sum::<u16>() + 2;
        let rows =
            crate::pages::layout::rows(right, &[needed.min(right.height.saturating_sub(10)), 1, 0]);
        let panel = Panel::card(Some("Principles"));
        let bg = panel.bg(t);
        let inner = panel.render(rows[0], buf, t);
        let mut y = inner.y;
        for (title, lines) in &wrapped {
            if y + 1 >= inner.bottom() {
                break;
            }
            buf.set_string(
                inner.x,
                y,
                title,
                t.primary().bg(bg).add_modifier(Modifier::BOLD),
            );
            y += 1;
            for l in lines {
                if y >= inner.bottom() {
                    break;
                }
                buf.set_string(inner.x, y, l, t.secondary().bg(bg));
                y += 1;
            }
            y += 1;
        }

        let panel = Panel::card(Some("State language"));
        let bg = panel.bg(t);
        let legend_area = Rect::new(rows[2].x, rows[2].y, rows[2].width, rows[2].height.min(10));
        let _ = &legend_area;
        let inner = panel.render(legend_area, buf, t);
        let legend: [(&str, &str, ratatui::style::Style); 7] = [
            ("▎", "focus", t.accent_fg()),
            ("░", "hover lifts the surface", t.secondary()),
            ("›", "current / chosen", t.primary()),
            ("✓", "checked", t.primary()),
            ("!", "error", t.error_fg()),
            ("▁", "editing: cursor + underline", t.primary()),
            ("○", "disabled: faint, no hover", t.faint()),
        ];
        for (i, (g, text, st)) in legend.iter().enumerate() {
            let y = inner.y + i as u16;
            if y >= inner.bottom() {
                break;
            }
            buf.set_string(inner.x, y, g, st.bg(bg));
            buf.set_string(inner.x + 3, y, text, t.secondary().bg(bg));
        }
    }

    fn handle(&mut self, _ev: &PageEvent, _cx: &mut PageCtx) -> Outcome {
        Outcome::Ignored
    }

    fn hints(&self, _focus: Option<WidgetId>) -> Vec<Hint> {
        vec![("[ ]", "Pages"), ("i", "Inspector")]
    }
}

/// The colour roles, each drawn by the library: status kinds
/// ([`Theme::status`], label in [`Theme::status_label`]), environments
/// ([`Theme::env`]), tones ([`Theme::tone_on`]), syntax
/// ([`Theme::syntax`]) and a link ([`Theme::link_style`]). Every text is
/// stepped up through [`Theme::legible`] where its hue would not read on
/// the card (skipped and faint).
fn render_roles(area: Rect, buf: &mut Buffer, t: &Theme) {
    if area.height < 4 {
        return;
    }
    let panel = Panel::card(Some("Roles"));
    let bg = panel.bg(t);
    let inner = panel.render(
        Rect::new(area.x, area.y, area.width, area.height.min(12)),
        buf,
        t,
    );
    let mut y = inner.y;
    let line =
        |buf: &mut Buffer, y: u16, label: &str, parts: &[(String, ratatui::style::Style)]| {
            if y >= inner.bottom() {
                return;
            }
            buf.set_string(inner.x, y, label, t.muted().bg(bg));
            let mut x = inner.x + 7;
            for (text, style) in parts {
                let w = junie_tui::ui::text::width(text) as u16;
                if x + w > inner.right() {
                    break;
                }
                buf.set_string(x, y, text, style.bg(bg));
                x += w;
            }
        };
    let gap = || (" ".to_owned(), t.primary());
    let status = |kinds: &[(StatusKind, &str)]| {
        let mut parts = Vec::new();
        for (kind, word) in kinds {
            let (glyph, style) = t.status(*kind);
            let label = t.status_label(*kind);
            let ink = t.legible(label.fg.unwrap_or(t.text_primary), bg);
            parts.push((glyph.to_owned(), style));
            parts.push((format!(" {word}"), label.fg(ink)));
            parts.push(gap());
        }
        parts
    };
    line(
        buf,
        y,
        "Status",
        &status(&[
            (StatusKind::Ok, "ok"),
            (StatusKind::Fail, "failed"),
            (StatusKind::Warn, "warn"),
            (StatusKind::Pending, "pending"),
        ]),
    );
    y += 1;
    line(
        buf,
        y,
        "",
        &status(&[
            (StatusKind::Running, "running"),
            (StatusKind::Queued, "queued"),
            (StatusKind::Info, "info"),
            (StatusKind::Skipped, "skipped"),
        ]),
    );
    y += 1;
    let env = |kinds: &[(EnvKind, &str)]| {
        let mut parts = Vec::new();
        for (kind, word) in kinds {
            let (_, style) = t.env(*kind);
            let glyph = t.glyphs.env(*kind);
            let text = if glyph.is_empty() {
                word.to_string()
            } else {
                format!("{glyph} {word}")
            };
            parts.push((text, style.fg(t.tone_on(kind.tone(), bg))));
            parts.push(gap());
        }
        parts
    };
    line(
        buf,
        y,
        "Env",
        &env(&[
            (EnvKind::Production, "production"),
            (EnvKind::Staging, "staging"),
        ]),
    );
    y += 1;
    line(
        buf,
        y,
        "",
        &env(&[
            (EnvKind::Development, "development"),
            (EnvKind::Local, "local"),
        ]),
    );
    y += 1;
    let tones = |list: &[(Tone, &str)]| {
        let mut parts = Vec::new();
        for (tone, word) in list {
            let fg = t.tone_on(*tone, bg);
            parts.push((word.to_string(), ratatui::style::Style::new().fg(fg)));
            parts.push(gap());
        }
        parts
    };
    line(
        buf,
        y,
        "Tone",
        &tones(&[
            (Tone::Normal, "normal"),
            (Tone::Secondary, "secondary"),
            (Tone::Muted, "muted"),
            (Tone::Faint, "faint"),
            (Tone::Active, "active"),
        ]),
    );
    y += 1;
    line(
        buf,
        y,
        "",
        &tones(&[
            (Tone::Info, "info"),
            (Tone::Success, "success"),
            (Tone::Warning, "warning"),
            (Tone::Error, "error"),
            (Tone::Danger, "danger"),
        ]),
    );
    y += 1;
    let code: Vec<(String, ratatui::style::Style)> = [
        ("let", SyntaxTone::Keyword),
        (" ", SyntaxTone::Plain),
        ("ok", SyntaxTone::Ident),
        (" = ", SyntaxTone::Operator),
        ("true", SyntaxTone::Literal),
        (", ", SyntaxTone::Punct),
        ("42", SyntaxTone::Number),
        (", ", SyntaxTone::Punct),
        ("\"text\"", SyntaxTone::Str),
        ("; ", SyntaxTone::Punct),
        ("// note", SyntaxTone::Comment),
    ]
    .into_iter()
    .map(|(text, tone)| (text.to_owned(), t.syntax(tone)))
    .collect();
    line(buf, y, "Code", &code);
    y += 1;
    line(
        buf,
        y,
        "Link",
        &[
            ("docs/colour.md".to_owned(), t.link_style(false)),
            ("  underlined on hover or focus".to_owned(), t.muted()),
        ],
    );
}
