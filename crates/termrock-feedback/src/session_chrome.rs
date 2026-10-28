use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::text::width;
use crate::theme::{FgStep, PaintStyle, Surface};
use crate::ui::Ui;

pub struct SessionMenu<'a> {
    pub label: &'a str,
    pub focused: bool,
}

pub struct SessionHint<'a> {
    pub key: &'a str,
    pub action: &'a str,
}

pub struct SessionChrome<'a> {
    pub brand: &'a str,
    pub menus: &'a [SessionMenu<'a>],
    pub crumb: &'a str,
    pub instance: &'a str,
    pub instances: &'a str,
    pub branch: &'a str,
    pub state: &'a str,
    pub account: &'a str,
    pub hints: &'a [SessionHint<'a>],
    pub overflow: &'a str,
    pub attached: &'a str,
}

struct Ink {
    canvas: Color,
    card: Color,
    primary: Color,
    secondary: Color,
    muted: Color,
    faint: Color,
    accent: Color,
    on_accent: Color,
}

pub fn paint_session_chrome(ui: &mut Ui<'_>, area: Rect, chrome: &SessionChrome<'_>) {
    if area.width < 8 || area.height < 3 {
        return;
    }
    let ink = {
        let theme = ui.theme_ref();
        Ink {
            canvas: theme.bg(Surface::Canvas),
            card: theme.bg(Surface::Elevated),
            primary: step(theme, FgStep::Primary),
            secondary: step(theme, FgStep::Secondary),
            muted: step(theme, FgStep::Muted),
            faint: step(theme, FgStep::Faint),
            accent: theme.color.accent,
            on_accent: theme.color.on_accent,
        }
    };
    let canvas = plain(ink.primary, ink.canvas);
    ui.fill(area, canvas);
    if area.height > 3 {
        ui.fill(
            Rect {
                x: area.x,
                y: area.y.saturating_add(1),
                width: area.width,
                height: area.height.saturating_sub(3),
            },
            canvas,
        );
    }
    paint_header(ui, area, chrome, &ink);
    paint_status(ui, area, chrome, &ink);
    paint_hints(ui, area, chrome, &ink);
}

fn plain(fg: Color, bg: Color) -> PaintStyle {
    PaintStyle::new()
        .fg(fg)
        .bg(bg)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM)
}

fn bold(fg: Color, bg: Color) -> PaintStyle {
    plain(fg, bg).add_modifier(Modifier::BOLD)
}

fn step(theme: &crate::theme::Theme, step: FgStep) -> Color {
    theme
        .color
        .fg
        .get(step.index())
        .copied()
        .unwrap_or(Color::Reset)
}

fn put(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let width = area.right().saturating_sub(x);
    if width == 0 || y >= area.bottom() {
        return;
    }
    ui.paint_str(Rect::new(x, y, width, 1), text, style);
}

fn paint_header(ui: &mut Ui<'_>, area: Rect, chrome: &SessionChrome<'_>, ink: &Ink) {
    let y = area.y;
    let brand = format!(" {} ", chrome.brand);
    let brand_style = bold(ink.on_accent, ink.accent);
    put(ui, area, area.x.saturating_add(1), y, &brand, brand_style);
    let mut x = area
        .x
        .saturating_add(1)
        .saturating_add(width(&brand))
        .saturating_add(1);
    let focused = bold(ink.primary, ink.canvas);
    let idle = plain(ink.secondary, ink.canvas);
    let gutter = plain(ink.accent, ink.canvas);
    for menu in chrome.menus {
        let label = format!(" {} ", menu.label);
        if menu.focused {
            put(ui, area, x, y, "▎", gutter);
            x = x.saturating_add(1);
            put(ui, area, x, y, &label, focused);
        } else {
            put(ui, area, x, y, &label, idle);
        }
        x = x.saturating_add(width(&label)).saturating_add(1);
    }
    let crumb = format!(" {}", chrome.crumb);
    let instance = format!(" {} ", chrome.instance);
    let cluster = width(&crumb)
        .saturating_add(1)
        .saturating_add(width(&instance))
        .saturating_add(1)
        .saturating_add(width(chrome.instances))
        .saturating_add(2);
    let cluster_x = area.right().saturating_sub(cluster);
    let bold = bold(ink.primary, ink.canvas);
    let muted = plain(ink.muted, ink.canvas);
    put(ui, area, cluster_x, y, &crumb, bold);
    let instance_x = cluster_x.saturating_add(width(&crumb)).saturating_add(1);
    put(ui, area, instance_x, y, &instance, muted);
    let count_x = instance_x
        .saturating_add(width(&instance))
        .saturating_add(1);
    put(ui, area, count_x, y, chrome.instances, muted);
}

fn paint_status(ui: &mut Ui<'_>, area: Rect, chrome: &SessionChrome<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(2);
    let row = plain(ink.primary, ink.card);
    ui.fill(
        Rect {
            x: area.x,
            y,
            width: area.width,
            height: 1,
        },
        row,
    );
    let bold = bold(ink.primary, ink.card);
    let muted = plain(ink.muted, ink.card);
    let branch_x = area.x.saturating_add(1);
    put(ui, area, branch_x, y, chrome.branch, bold);
    put(
        ui,
        area,
        branch_x
            .saturating_add(width(chrome.branch))
            .saturating_add(3),
        y,
        chrome.state,
        muted,
    );
    let account_x = area
        .right()
        .saturating_sub(1)
        .saturating_sub(width(chrome.account));
    put(ui, area, account_x, y, chrome.account, muted);
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, chrome: &SessionChrome<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    let mut x = area.x.saturating_add(1);
    let key_style = bold(ink.primary, ink.canvas);
    let action_style = plain(ink.muted, ink.canvas);
    for hint in chrome.hints {
        put(ui, area, x, y, hint.key, key_style);
        x = x.saturating_add(width(hint.key)).saturating_add(1);
        put(ui, area, x, y, hint.action, action_style);
        x = x.saturating_add(width(hint.action)).saturating_add(2);
    }
    put(
        ui,
        area,
        x,
        y,
        chrome.overflow,
        plain(ink.faint, ink.canvas),
    );
    let attached_x = area
        .right()
        .saturating_sub(1)
        .saturating_sub(width(chrome.attached));
    put(
        ui,
        area,
        attached_x,
        y,
        chrome.attached,
        plain(ink.secondary, ink.canvas),
    );
}
