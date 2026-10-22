//! Attached-session chrome: menu row, empty body, status row, hint row.
//!
//! Positions follow the label widths and the screen edges. The body keeps
//! the canvas ink. Callers pass the words; this does not store a frame.

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

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;
    use ratatui_core::layout::Rect;
    use ratatui_core::style::{Color, Modifier};

    use super::{SessionChrome, SessionHint, SessionMenu, paint_session_chrome};
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::Theme;

    fn chrome<'a>() -> SessionChrome<'a> {
        SessionChrome {
            brand: "jackin❯",
            menus: &[
                SessionMenu {
                    label: "File",
                    focused: true,
                },
                SessionMenu {
                    label: "Edit",
                    focused: false,
                },
                SessionMenu {
                    label: "View",
                    focused: false,
                },
                SessionMenu {
                    label: "Session",
                    focused: false,
                },
                SessionMenu {
                    label: "Help",
                    focused: false,
                },
            ],
            crumb: "› backend",
            instance: "jackin-data-pipeline-e0e0",
            instances: "3 instances",
            branch: "main",
            state: "clean",
            account: "no account · shell",
            hints: &[
                SessionHint {
                    key: "Ctrl+B",
                    action: "Pane commands…",
                },
                SessionHint {
                    key: "F10",
                    action: "Menu",
                },
                SessionHint {
                    key: "Ctrl+\\",
                    action: "Palette",
                },
                SessionHint {
                    key: "Alt+Shift+↑↓←→",
                    action: "Resize",
                },
            ],
            overflow: "…",
            attached: "Attached to  · tabs and panes restored",
        }
    }

    #[test]
    fn attached_shell_places_the_menu_status_and_hints() {
        let area = Rect::new(0, 0, 120, 40);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, screen| {
                paint_session_chrome(ui, screen, &chrome());
            })
            .commit_presented();

        let brand = buffer.cell((1, 0)).expect("brand");
        assert_eq!(brand.symbol(), " ");
        assert_eq!(brand.fg, Color::Rgb(0x19, 0x19, 0x1c));
        assert_eq!(brand.bg, Color::Rgb(0x48, 0xe0, 0x54));
        assert!(brand.modifier.contains(Modifier::BOLD));

        let gutter = buffer.cell((11, 0)).expect("gutter");
        assert_eq!(gutter.symbol(), "▎");
        assert_eq!(gutter.fg, Color::Rgb(0x48, 0xe0, 0x54));
        assert_eq!(gutter.bg, Color::Rgb(0x00, 0x00, 0x00));

        let file = buffer.cell((13, 0)).expect("file");
        assert_eq!(file.symbol(), "F");
        assert_eq!(file.fg, Color::Rgb(0xff, 0xff, 0xff));
        assert!(file.modifier.contains(Modifier::BOLD));

        let edit = buffer.cell((20, 0)).expect("edit");
        assert_eq!(edit.symbol(), "E");
        assert_eq!(edit.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
        assert!(!edit.modifier.contains(Modifier::BOLD));

        let instance = buffer.cell((80, 0)).expect("instance");
        assert_eq!(instance.symbol(), "j");
        assert_eq!(instance.fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(instance.bg, Color::Rgb(0x00, 0x00, 0x00));

        let body = buffer.cell((0, 1)).expect("body");
        assert_eq!(body.symbol(), " ");
        assert_eq!(body.fg, Color::Rgb(0xff, 0xff, 0xff));
        assert_eq!(body.bg, Color::Rgb(0x00, 0x00, 0x00));

        let branch = buffer.cell((1, 38)).expect("branch");
        assert_eq!(branch.symbol(), "m");
        assert_eq!(branch.bg, Color::Rgb(0x18, 0x18, 0x1b));
        assert!(branch.modifier.contains(Modifier::BOLD));

        let state = buffer.cell((8, 38)).expect("state");
        assert_eq!(state.symbol(), "c");
        assert_eq!(state.fg, Color::Rgb(0x80, 0x80, 0x80));
        assert_eq!(state.bg, Color::Rgb(0x18, 0x18, 0x1b));

        let key = buffer.cell((1, 39)).expect("key");
        assert_eq!(key.symbol(), "C");
        assert!(key.modifier.contains(Modifier::BOLD));

        let attached = buffer.cell((81, 39)).expect("attached");
        assert_eq!(attached.symbol(), "A");
        assert_eq!(attached.fg, Color::Rgb(0xb3, 0xb3, 0xb3));
    }
}
