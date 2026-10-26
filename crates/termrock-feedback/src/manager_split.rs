use ratatui_core::layout::Rect;
use ratatui_core::style::{Color, Modifier};

use crate::id::{Id, Part};
use crate::response::StateFlags;
use crate::text::width;
use crate::theme::{
    Family, FgStep, ManagerPresentationProfile, MonoRule, PaintStyle, Role, StyleDefaults,
    StylePatch, Surface, Variant,
};
use crate::ui::Ui;
use termrock_controls::Brand;

const SELECT_BG: Color = Color::Rgb(0x17, 0x3f, 0x1c);
const EXPANDED_BG: Color = Color::Rgb(0x30, 0x30, 0x36);
const RULE: Color = Color::Rgb(0x3a, 0x3a, 0x3a);
const BADGE: Color = Color::Rgb(0x73, 0x73, 0x73);
const DANGER_BUTTON: Color = Color::Rgb(0xff, 0x6b, 0x6b);
const INDEX_ERROR: Color = Color::Rgb(0xf0, 0x5d, 0x5d);
const STALE: Color = Color::Rgb(0xf5, 0x9e, 0x09);
const LABEL_BUDGET: u16 = 28;

const MANAGER_FAMILY: Family = Family::custom("termrock.feedback.manager_split");
const PRIMARY_PART: Part = Part::custom("termrock.manager.primary");
const SECONDARY_PART: Part = Part::custom("termrock.manager.secondary");
const MUTED_PART: Part = Part::custom("termrock.manager.muted");
const ACCENT_PART: Part = Part::custom("termrock.manager.accent");
const BORDER_PART: Part = Part::custom("termrock.manager.border");
const SEAM_PART: Part = Part::custom("termrock.manager.seam");
const SURFACE_PART: Part = Part::custom("termrock.manager.surface_fill");
const DANGER_PART: Part = Part::custom("termrock.manager.danger");
const WARNING_PART: Part = Part::custom("termrock.manager.warning");

const MONO_RULES: &[MonoRule] = &[
    (
        SECONDARY_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        MUTED_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        ACCENT_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        BORDER_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        SEAM_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        DANGER_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
    (
        WARNING_PART,
        StateFlags::empty(),
        StylePatch::new().set_fg(Role::Fg(FgStep::Primary)),
    ),
];

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SplitMode {
    Empty,

    Saved,

    Expanded,
}

pub struct SplitMount<'a> {
    pub source: &'a str,
    pub mode: &'a str,
    pub isolation: &'a str,
}

pub struct SplitInstance<'a> {
    pub id: &'a str,
    pub role: &'a str,
    pub agent: &'a str,
    pub status: &'a str,
    pub live: bool,
    pub dirty: bool,
}

pub struct SplitWorkspace<'a> {
    pub name: &'a str,

    pub expandable: bool,
}

#[derive(Clone, Copy)]
pub struct SplitAlert<'a> {
    pub index_error: Option<&'a str>,
    pub daemon_stale: bool,
}

pub struct ManagerSplit<'a> {
    pub brand_id: Id,
    pub mode: SplitMode,
    pub menus: &'a [&'a str],
    pub workspace_name: &'a str,
    pub workdir: &'a str,
    pub cwd_display: &'a str,
    pub mounts: &'a [SplitMount<'a>],
    pub role_line: &'a str,
    pub env_line: &'a str,
    pub account_lines: &'a [&'a str],
    pub policy_line: &'a str,
    pub workspaces: &'a [SplitWorkspace<'a>],
    pub instances: &'a [SplitInstance<'a>],
    pub running: u16,
    pub live_here: u16,
    pub daemon_ago: &'a str,
    pub alert: Option<SplitAlert<'a>>,
}

#[derive(Clone, Copy)]
enum InkBackground {
    Canvas,
    Card,
    Overlay,
    AccentTint,
    Expanded,
}

#[derive(Clone, Copy)]
struct SemanticForeground {
    canvas: PaintStyle,
    card: PaintStyle,
    overlay: PaintStyle,
    accent_tint: PaintStyle,
    expanded: PaintStyle,
    preserve_background_role: bool,
}

impl SemanticForeground {
    const fn on(self, background: InkBackground) -> ForegroundStyle {
        let style = match background {
            InkBackground::Canvas => self.canvas,
            InkBackground::Card => self.card,
            InkBackground::Overlay => self.overlay,
            InkBackground::AccentTint => self.accent_tint,
            InkBackground::Expanded => self.expanded,
        };
        ForegroundStyle::Semantic {
            style,
            preserve_background_role: self.preserve_background_role,
        }
    }
}

#[derive(Clone, Copy)]
enum ForegroundStyle {
    Literal(Color),
    Semantic {
        style: PaintStyle,
        preserve_background_role: bool,
    },
}

impl From<Color> for ForegroundStyle {
    fn from(color: Color) -> Self {
        Self::Literal(color)
    }
}

impl From<PaintStyle> for ForegroundStyle {
    fn from(style: PaintStyle) -> Self {
        Self::Semantic {
            style,
            preserve_background_role: false,
        }
    }
}

impl ForegroundStyle {
    fn into_paint_style(self, background: Color) -> PaintStyle {
        match self {
            Self::Literal(color) => PaintStyle::new().fg(color).bg(background),
            Self::Semantic {
                style,
                preserve_background_role: true,
            } => style,
            Self::Semantic {
                style,
                preserve_background_role: false,
            } => PaintStyle::new().with_fg_from(style).bg(background),
        }
    }

    fn paint_style(self) -> PaintStyle {
        match self {
            Self::Literal(color) => PaintStyle::new().fg(color),
            Self::Semantic { style, .. } => style,
        }
    }
}

struct Ink {
    canvas: Color,
    card: Color,
    canvas_surface_fill: PaintStyle,
    overlay_surface_fill: PaintStyle,
    primary: SemanticForeground,
    secondary: SemanticForeground,
    muted: SemanticForeground,
    accent: SemanticForeground,
    live: SemanticForeground,
    idle: SemanticForeground,
    border: SemanticForeground,
    button: Color,
    select_bg: Color,
    expanded_bg: Color,
    rule: SemanticForeground,
    badge: SemanticForeground,
    danger_button: SemanticForeground,
    index_error: SemanticForeground,
    stale: SemanticForeground,
    paper_semantic: bool,
    restore_saved_layout: bool,
}

pub fn paint_manager_split(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>) {
    paint_manager_split_inner(ui, area, frame, false);
}

fn paint_manager_split_inner(
    ui: &mut Ui<'_>,
    area: Rect,
    frame: &ManagerSplit<'_>,
    detail_drawer: bool,
) {
    if area.width < 80 || area.height < 20 {
        return;
    }
    let restore_saved_layout = (manager_semantic_mono(ui) || manager_semantic_paper(ui))
        && frame.mode == SplitMode::Saved
        && frame.alert.is_none()
        && !detail_drawer;
    let ink = ink_from(ui, restore_saved_layout);
    ui.fill(
        area,
        plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
    );
    let _ = Brand::new(frame.brand_id, "jackin❯")
        .draw(ui, Rect::new(area.x.saturating_add(1), area.y, 9, 1));
    paint_menus(ui, area, frame.menus, &ink);
    paint_identity(ui, area, frame, &ink);
    ui.fill(
        Rect::new(
            40,
            area.y.saturating_add(2),
            area.width.saturating_sub(41),
            36,
        ),
        plain(ink.primary.on(InkBackground::Card), ink.card),
    );
    paint_box(ui, area, frame, &ink);
    paint_rows(ui, area, frame, &ink);
    paint_detail(ui, area, frame, &ink);
    paint_buttons(ui, area, frame, &ink);
    paint_hints(ui, area, frame, &ink);
}

fn leading_selection_style(ink: &Ink) -> PaintStyle {
    if ink.paper_semantic {
        let selected = ink.primary.on(InkBackground::AccentTint).paint_style();
        PaintStyle::new()
            .with_fg_from_bg(selected)
            .with_bg_from(selected)
            .add_modifier(Modifier::BOLD)
    } else {
        bold(ink.select_bg, ink.select_bg)
    }
}

pub fn paint_manager_detail_drawer(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>) {
    paint_manager_split_inner(ui, area, frame, true);
    let ink = ink_from(ui, false);
    let rule_style = plain(ink.rule.on(InkBackground::Canvas), ink.canvas);

    let summary = format!(" {} running ", frame.running);
    let (line, badge_x) = box_top(&summary);
    let chars: Vec<char> = line.chars().collect();
    let tail = chars.len().saturating_sub(2);
    let dashes_end = usize::from(badge_x).saturating_sub(1);
    let seg = |from: usize, to: usize| chars[from..to].iter().collect::<String>();
    if chars.len() >= 2 {
        put(ui, area, 1, 2, &seg(0, 2), rule_style);
        if tail >= 14 {
            put(
                ui,
                area,
                3,
                2,
                &seg(2, 14),
                plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
            );
        }
        if dashes_end > 14 && dashes_end <= tail {
            put(ui, area, 15, 2, &seg(14, dashes_end), rule_style);
        }
        let tail_x = u16::try_from(1 + tail).unwrap_or(36);
        put(ui, area, tail_x, 2, &seg(tail, chars.len()), rule_style);
    }
    for y in 3..37 {
        put(ui, area, 1, y, "│", rule_style);
        put(ui, area, 37, y, "│", rule_style);
    }
    put(
        ui,
        area,
        1,
        37,
        "╰───────────────────────────────────╯",
        rule_style,
    );

    put(
        ui,
        area,
        41,
        2,
        "▎",
        plain(ink.accent.on(InkBackground::Card), ink.card),
    );
    put(
        ui,
        area,
        42,
        2,
        &format!("Current directory · {}", frame.workspace_name),
        bold(ink.primary.on(InkBackground::Card), ink.card),
    );

    let leading_selection = leading_selection_style(&ink);
    put(ui, area, 3, 3, " ", leading_selection);
    put(
        ui,
        area,
        5,
        3,
        " ",
        plain(ink.primary.on(InkBackground::AccentTint), ink.select_bg),
    );

    let key = bold(ink.primary.on(InkBackground::Canvas), ink.canvas);
    let hint = plain(ink.muted.on(InkBackground::Canvas), ink.canvas);
    let y = 39;
    ui.fill(
        Rect::new(1, y, 89, 1),
        plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
    );
    put(ui, area, 1, y, "↑↓", key);
    put(ui, area, 4, y, "Move", hint);
    put(ui, area, 10, y, "Tab", key);
    put(ui, area, 14, y, "Actions", hint);
    put(ui, area, 23, y, "Esc", key);
    put(ui, area, 27, y, "Back", hint);
}

fn manager_semantic_mono(ui: &Ui<'_>) -> bool {
    let theme = ui.theme_ref();
    theme.capability.color == crate::theme::ColorLevel::Mono
        && theme.manager_presentation_profile() == ManagerPresentationProfile::Junie
}

fn manager_semantic_paper(ui: &Ui<'_>) -> bool {
    let theme = ui.theme_ref();
    matches!(
        theme.capability.color,
        crate::theme::ColorLevel::Mono | crate::theme::ColorLevel::TrueColor
    ) && theme.manager_presentation_profile() == ManagerPresentationProfile::Paper
}

fn ink_from(ui: &mut Ui<'_>, restore_saved_layout: bool) -> Ink {
    let (
        semantic_mono,
        semantic_paper,
        canvas,
        card,
        button,
        primary_fallback,
        secondary_fallback,
        muted_fallback,
        accent_fallback,
        border_fallback,
        select_bg_fallback,
        expanded_bg,
        rule_fallback,
        badge_fallback,
        danger_button_fallback,
        index_error_fallback,
        stale_fallback,
    ) = {
        let theme = ui.theme_ref();
        let level = theme.capability.color;
        let profile = theme.manager_presentation_profile();
        let downgrade = |color: Color| crate::theme::downgrade_color(color, level);
        (
            level == crate::theme::ColorLevel::Mono && profile == ManagerPresentationProfile::Junie,
            matches!(
                level,
                crate::theme::ColorLevel::Mono | crate::theme::ColorLevel::TrueColor
            ) && profile == ManagerPresentationProfile::Paper,
            theme.bg(Surface::Canvas),
            theme.bg(Surface::Surface),
            theme.bg(Surface::Overlay),
            step(theme, FgStep::Primary),
            step(theme, FgStep::Secondary),
            step(theme, FgStep::Muted),
            theme.color.accent,
            theme.color.border_strong,
            downgrade(SELECT_BG),
            downgrade(EXPANDED_BG),
            downgrade(RULE),
            downgrade(BADGE),
            downgrade(DANGER_BUTTON),
            downgrade(INDEX_ERROR),
            downgrade(STALE),
        )
    };
    let canvas_surface_fill = semantic_surface_fill(ui, semantic_paper, Surface::Canvas, canvas);
    let overlay_surface_fill = semantic_surface_fill(ui, semantic_paper, Surface::Overlay, button);
    let primary = semantic_foreground(
        ui,
        semantic_mono,
        semantic_paper,
        PRIMARY_PART,
        Role::Fg(FgStep::Primary),
        primary_fallback,
    );
    let secondary = semantic_foreground(
        ui,
        semantic_mono,
        semantic_paper,
        SECONDARY_PART,
        Role::Fg(FgStep::Secondary),
        secondary_fallback,
    );
    let muted = semantic_foreground(
        ui,
        semantic_mono,
        semantic_paper,
        MUTED_PART,
        Role::Fg(FgStep::Muted),
        muted_fallback,
    );
    let accent = semantic_foreground(
        ui,
        semantic_mono,
        semantic_paper,
        ACCENT_PART,
        Role::Accent,
        accent_fallback,
    );
    let live = if semantic_paper {
        semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            PRIMARY_PART,
            Role::Fg(FgStep::Primary),
            accent_fallback,
        )
    } else {
        accent
    };
    let idle = if semantic_paper {
        semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            SECONDARY_PART,
            Role::Fg(FgStep::Secondary),
            muted_fallback,
        )
    } else {
        muted
    };
    let border = semantic_foreground(
        ui,
        semantic_mono,
        semantic_paper,
        BORDER_PART,
        Role::BorderStrong,
        border_fallback,
    );
    let badge = if semantic_paper {
        semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            BORDER_PART,
            Role::BorderStrong,
            badge_fallback,
        )
    } else {
        semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            MUTED_PART,
            Role::Fg(FgStep::Muted),
            badge_fallback,
        )
    };
    let rule = semantic_foreground(
        ui,
        semantic_mono,
        semantic_paper,
        SEAM_PART,
        Role::BorderSubtle,
        rule_fallback,
    );
    let select_bg = if semantic_paper {
        primary
            .on(InkBackground::AccentTint)
            .paint_style()
            .as_style()
            .bg
            .unwrap_or(select_bg_fallback)
    } else {
        select_bg_fallback
    };
    Ink {
        canvas,
        card,
        canvas_surface_fill,
        overlay_surface_fill,
        primary,
        secondary,
        muted,
        accent,
        live,
        idle,
        border,
        button,
        select_bg,
        expanded_bg,
        rule,
        badge,
        danger_button: semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            DANGER_PART,
            Role::Danger,
            danger_button_fallback,
        ),
        index_error: semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            DANGER_PART,
            Role::Danger,
            index_error_fallback,
        ),
        stale: semantic_foreground(
            ui,
            semantic_mono,
            semantic_paper,
            WARNING_PART,
            Role::Warning,
            stale_fallback,
        ),
        paper_semantic: semantic_paper,
        restore_saved_layout,
    }
}

fn semantic_surface_fill(
    ui: &mut Ui<'_>,
    semantic_paper: bool,
    surface: Surface,
    fallback: Color,
) -> PaintStyle {
    if !semantic_paper {
        return PaintStyle::new().fg(fallback).bg(fallback);
    }
    let role = Role::Surface(surface);
    ui.with_surface(surface, |ui| {
        ui.style_defaults(
            MANAGER_FAMILY,
            Variant::DEFAULT,
            SURFACE_PART,
            StateFlags::empty(),
            StyleDefaults::new(StylePatch::new().set_fg(role).set_bg(role)),
            None,
        )
        .style
    })
}

fn semantic_foreground(
    ui: &mut Ui<'_>,
    semantic_mono: bool,
    semantic_paper: bool,
    part: Part,
    role: Role,
    fallback: Color,
) -> SemanticForeground {
    let semantic = semantic_mono || semantic_paper;
    let mut resolve = |background: InkBackground| {
        if !semantic {
            return PaintStyle::new().fg(fallback);
        }
        let (surface, background_role) = match background {
            InkBackground::Canvas => (Surface::Canvas, Role::Surface(Surface::Canvas)),
            InkBackground::Card => (Surface::Surface, Role::Surface(Surface::Surface)),
            InkBackground::Overlay => (Surface::Overlay, Role::Surface(Surface::Overlay)),
            InkBackground::AccentTint => (Surface::Canvas, Role::AccentTint),
            InkBackground::Expanded => (Surface::Canvas, Role::Custom(EXPANDED_BG)),
        };
        ui.with_surface(surface, |ui| {
            let defaults =
                StyleDefaults::new(StylePatch::new().set_fg(role).set_bg(background_role));
            let defaults = if semantic_mono {
                defaults.mono(MONO_RULES)
            } else {
                defaults
            };
            ui.style_defaults(
                MANAGER_FAMILY,
                Variant::DEFAULT,
                part,
                StateFlags::empty(),
                defaults,
                None,
            )
            .style
        })
    };
    SemanticForeground {
        canvas: resolve(InkBackground::Canvas),
        card: resolve(InkBackground::Card),
        overlay: resolve(InkBackground::Overlay),
        accent_tint: resolve(InkBackground::AccentTint),
        expanded: resolve(InkBackground::Expanded),
        preserve_background_role: semantic_paper,
    }
}

fn step(theme: &crate::theme::Theme, which: FgStep) -> Color {
    theme
        .color
        .fg
        .get(which.index())
        .copied()
        .unwrap_or(Color::Reset)
}

fn plain(fg: impl Into<ForegroundStyle>, bg: Color) -> PaintStyle {
    fg.into()
        .into_paint_style(bg)
        .remove_modifier(Modifier::BOLD | Modifier::ITALIC | Modifier::UNDERLINED | Modifier::DIM)
}

fn bold(fg: impl Into<ForegroundStyle>, bg: Color) -> PaintStyle {
    plain(fg, bg).add_modifier(Modifier::BOLD)
}

fn put(ui: &mut Ui<'_>, area: Rect, x: u16, y: u16, text: &str, style: PaintStyle) {
    let room = area.right().saturating_sub(x);
    if room == 0 || y >= area.bottom() || text.is_empty() {
        return;
    }
    ui.paint_str(Rect::new(x, y, room, 1), text, style);
}

fn paint_menus(ui: &mut Ui<'_>, area: Rect, menus: &[&str], ink: &Ink) {
    let style = plain(ink.secondary.on(InkBackground::Canvas), ink.canvas);
    let mut x = 12u16;
    for label in menus {
        let text = format!(" {label} ");
        put(ui, area, x, area.y, &text, style);
        x = x.saturating_add(width(&text)).saturating_add(1);
    }
}

fn paint_identity(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    if frame.mode == SplitMode::Saved
        && let Some(alert) = frame.alert
    {
        paint_degraded_identity(ui, area, alert, ink);
        return;
    }
    let y = area.y;
    let (crumb, crumb_x, inside_x, running_x, running) = match frame.mode {
        SplitMode::Empty => ("Workspaces", 72, 84, 106, "no instances".to_owned()),
        SplitMode::Saved if ink.restore_saved_layout => (
            "Workspaces",
            76,
            88,
            110,
            format!("{} running", frame.running),
        ),
        SplitMode::Saved => (
            "Workspaces",
            73,
            85,
            108,
            format!(" {} running", frame.running),
        ),
        SplitMode::Expanded => (
            "Workspaces",
            73,
            85,
            108,
            format!(" {} running", frame.running),
        ),
    };
    let crumb = if frame.mode == SplitMode::Expanded {
        format!("Workspaces › {}", frame.workspace_name)
    } else {
        crumb.to_owned()
    };
    let crumb_x = if frame.mode == SplitMode::Expanded {
        53
    } else {
        crumb_x
    };
    put(
        ui,
        area,
        crumb_x,
        y,
        &crumb,
        plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
    );
    put(
        ui,
        area,
        inside_x,
        y,
        "inside the Construct",
        plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
    );
    if frame.mode != SplitMode::Empty && !ink.restore_saved_layout {
        put(
            ui,
            area,
            107,
            y,
            "●",
            plain(ink.accent.on(InkBackground::Canvas), ink.canvas),
        );
    }
    let running_style = if frame.mode == SplitMode::Empty || ink.paper_semantic {
        plain(ink.muted.on(InkBackground::Canvas), ink.canvas)
    } else {
        plain(ink.secondary.on(InkBackground::Canvas), ink.canvas)
    };
    put(ui, area, running_x, y, &running, running_style);
}

fn paint_degraded_identity(ui: &mut Ui<'_>, area: Rect, alert: SplitAlert<'_>, ink: &Ink) {
    let y = area.y;
    let secondary = plain(ink.secondary.on(InkBackground::Canvas), ink.canvas);
    let mut x = 41u16;
    let crumb = "Workspaces";
    put(ui, area, x, y, crumb, secondary);
    x = x.saturating_add(width(crumb)).saturating_add(2);
    let inside = "inside the Construct";
    put(ui, area, x, y, inside, secondary);
    x = x.saturating_add(width(inside)).saturating_add(2);
    if let Some(error) = alert.index_error.filter(|error| !error.is_empty()) {
        let text = format!("! {error}");
        put(
            ui,
            area,
            x,
            y,
            &text,
            plain(ink.index_error.on(InkBackground::Canvas), ink.canvas),
        );
        x = x.saturating_add(width(&text)).saturating_add(2);
    }
    if alert.daemon_stale {
        put(
            ui,
            area,
            x,
            y,
            "▲ daemon stale",
            plain(ink.stale.on(InkBackground::Canvas), ink.canvas),
        );
    }
}

fn box_top(summary: &str) -> (String, u16) {
    let prefix = "╭─ Workspaces ";
    let suffix = "─╮";
    let width = 37usize;
    let dashes = width
        .saturating_sub(prefix.chars().count())
        .saturating_sub(summary.chars().count())
        .saturating_sub(suffix.chars().count());
    let line = format!("{prefix}{}{summary}{suffix}", "─".repeat(dashes));
    let badge_x = 1 + prefix.chars().count() as u16 + dashes as u16;
    (line, badge_x)
}

fn paint_box(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let summary = match frame.mode {
        SplitMode::Empty => " no instances ".to_owned(),
        SplitMode::Saved | SplitMode::Expanded => format!(" {} running ", frame.running),
    };
    let (line, badge_x) = box_top(&summary);
    let border = plain(ink.border.on(InkBackground::Canvas), ink.canvas);
    let seam = if ink.paper_semantic {
        plain(ink.rule.on(InkBackground::Canvas), ink.canvas)
    } else {
        border
    };
    put(ui, area, 1, 2, &line, border);
    put(
        ui,
        area,
        3,
        2,
        " Workspaces ",
        bold(ink.primary.on(InkBackground::Canvas), ink.canvas),
    );
    put(
        ui,
        area,
        badge_x,
        2,
        &summary,
        plain(ink.badge.on(InkBackground::Canvas), ink.canvas),
    );
    for y in 3..37 {
        put(ui, area, 1, y, "│", border);
        put(ui, area, 37, y, "│", border);
        put(ui, area, 39, y, "│", seam);
    }
    put(
        ui,
        area,
        1,
        37,
        "╰───────────────────────────────────╯",
        border,
    );
    put(ui, area, 39, 2, "│", seam);
    put(ui, area, 39, 37, "│", seam);
}

fn paint_rows(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let mut y = 3u16;
    match frame.mode {
        SplitMode::Empty => {
            paint_current(ui, area, y, true, ink);
            y = y.saturating_add(1);
            put(ui, area, 3, y, " ", ink.canvas_surface_fill);
            let mut action = "+ New workspace".to_owned();
            while action.chars().count() < 28 {
                action.push(' ');
            }
            put(
                ui,
                area,
                7,
                y,
                &action,
                plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
            );
        }
        SplitMode::Saved => {
            paint_current(ui, area, y, true, ink);
            y = y.saturating_add(1);
            let gutter = if ink.restore_saved_layout { "▎" } else { " " };
            for workspace in frame.workspaces {
                put(ui, area, 3, y, gutter, ink.canvas_surface_fill);
                let mark = if workspace.expandable { "▸" } else { " " };
                put(
                    ui,
                    area,
                    5,
                    y,
                    mark,
                    plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
                );
                let label = crate::text::truncate_middle(workspace.name, LABEL_BUDGET);
                put(
                    ui,
                    area,
                    7,
                    y,
                    &label,
                    plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
                );
                y = y.saturating_add(1);
            }
            put(ui, area, 3, y, gutter, ink.canvas_surface_fill);
            let mut action = "+ New workspace".to_owned();
            while action.chars().count() < 28 {
                action.push(' ');
            }
            put(
                ui,
                area,
                7,
                y,
                &action,
                plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
            );
        }
        SplitMode::Expanded => {
            paint_current(ui, area, y, false, ink);
            y = y.saturating_add(1);
            paint_expanded_workspace(ui, area, y, frame.workspace_name, ink);
            y = y.saturating_add(1);
            for instance in frame.instances {
                paint_child(ui, area, y, instance, ink);
                y = y.saturating_add(1);
            }
            for workspace in frame
                .workspaces
                .iter()
                .filter(|workspace| workspace.name != frame.workspace_name)
            {
                put(ui, area, 3, y, " ", ink.canvas_surface_fill);
                put(
                    ui,
                    area,
                    5,
                    y,
                    "▸",
                    plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
                );
                put(
                    ui,
                    area,
                    7,
                    y,
                    workspace.name,
                    plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
                );
                y = y.saturating_add(1);
            }
            put(ui, area, 3, y, " ", ink.canvas_surface_fill);
            let mut action = "+ New workspace".to_owned();
            while action.chars().count() < 28 {
                action.push(' ');
            }
            put(
                ui,
                area,
                7,
                y,
                &action,
                plain(ink.secondary.on(InkBackground::Canvas), ink.canvas),
            );
        }
    }
}

fn paint_current(ui: &mut Ui<'_>, area: Rect, y: u16, selected: bool, ink: &Ink) {
    if selected {
        ui.fill(
            Rect::new(3, y, 33, 1),
            bold(ink.primary.on(InkBackground::AccentTint), ink.select_bg),
        );
        put(
            ui,
            area,
            3,
            y,
            "▎",
            bold(ink.accent.on(InkBackground::AccentTint), ink.select_bg),
        );
        let mut label = "Current directory".to_owned();
        while label.chars().count() < 28 {
            label.push(' ');
        }
        put(
            ui,
            area,
            7,
            y,
            &label,
            bold(ink.accent.on(InkBackground::AccentTint), ink.select_bg),
        );
    } else {
        put(ui, area, 3, y, " ", ink.canvas_surface_fill);
        put(
            ui,
            area,
            7,
            y,
            "Current directory",
            plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
        );
    }
}

fn paint_expanded_workspace(ui: &mut Ui<'_>, area: Rect, y: u16, name: &str, ink: &Ink) {
    ui.fill(
        Rect::new(3, y, 33, 1),
        bold(ink.primary.on(InkBackground::Expanded), ink.expanded_bg),
    );
    put(
        ui,
        area,
        3,
        y,
        "▎",
        bold(ink.accent.on(InkBackground::Expanded), ink.expanded_bg),
    );
    put(
        ui,
        area,
        5,
        y,
        "▾",
        plain(ink.secondary.on(InkBackground::Expanded), ink.expanded_bg),
    );
    let mut label = name.to_owned();
    while label.chars().count() < 28 {
        label.push(' ');
    }
    put(
        ui,
        area,
        7,
        y,
        &label,
        bold(ink.accent.on(InkBackground::Expanded), ink.expanded_bg),
    );
    put(
        ui,
        area,
        35,
        y,
        " ",
        bold(ink.primary.on(InkBackground::Expanded), ink.expanded_bg),
    );
}

fn paint_child(ui: &mut Ui<'_>, area: Rect, y: u16, instance: &SplitInstance<'_>, ink: &Ink) {
    let prefix = format!(" {}  ", instance.id);
    let rest = format!("{} · {}", instance.role, instance.agent);
    let text = if instance.live {
        let body = middle_ellipsis(&prefix, &rest, 27);
        format!("{body:<29}")
    } else {
        format!("{prefix}{rest}")
    };
    put(ui, area, 3, y, " ", ink.canvas_surface_fill);
    let mark = if instance.live { "◉" } else { "◌" };
    let mark_style = if instance.live {
        plain(ink.live.on(InkBackground::Canvas), ink.canvas)
    } else {
        plain(ink.idle.on(InkBackground::Canvas), ink.canvas)
    };
    put(ui, area, 7, y, mark, mark_style);
    put(
        ui,
        area,
        8,
        y,
        &text,
        plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
    );
    if instance.dirty {
        put(
            ui,
            area,
            34,
            y,
            "•",
            plain(ink.stale.on(InkBackground::Canvas), ink.canvas),
        );
    }
}

fn middle_ellipsis(prefix: &str, rest: &str, budget: usize) -> String {
    let prefix_len = prefix.chars().count();
    if prefix_len.saturating_add(rest.chars().count()) <= budget {
        return format!("{prefix}{rest}");
    }
    let room = budget.saturating_sub(prefix_len);
    let tail = 8.min(room.saturating_sub(1));
    let head = room.saturating_sub(1).saturating_sub(tail);
    let head_s: String = rest.chars().take(head).collect();
    let tail_s: String = rest
        .chars()
        .rev()
        .take(tail)
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    format!("{prefix}{head_s}…{tail_s}")
}

fn paint_detail(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let card = plain(ink.primary.on(InkBackground::Card), ink.card);
    let muted = plain(ink.muted.on(InkBackground::Card), ink.card);
    let badge = plain(ink.badge.on(InkBackground::Card), ink.card);
    let secondary = plain(ink.secondary.on(InkBackground::Card), ink.card);
    match frame.mode {
        SplitMode::Empty => {
            put(ui, area, 42, 2, "Current directory", secondary);
            put(ui, area, 108, 2, "not saved", badge);
            if !ink.restore_saved_layout {
                put(
                    ui,
                    area,
                    42,
                    3,
                    &"─".repeat(75),
                    plain(ink.rule.on(InkBackground::Card), ink.card),
                );
            }
            put(
                ui,
                area,
                60,
                17,
                "Create a workspace from this directory.",
                muted,
            );
            put(
                ui,
                area,
                49,
                19,
                &format!(
                    "{} is mounted at its own path inside the",
                    frame.cwd_display
                ),
                badge,
            );
            put(
                ui,
                area,
                45,
                20,
                "Construct. Enter launches with defaults; n creates a saved workspace.",
                badge,
            );
        }
        SplitMode::Saved | SplitMode::Expanded => {
            let (title, badge_text, badge_x) = if frame.mode == SplitMode::Expanded {
                (
                    frame.workspace_name.to_owned(),
                    format!("saved workspace · {} running", frame.live_here),
                    90,
                )
            } else {
                (
                    format!("Current directory · {}", frame.workspace_name),
                    "saved workspace".to_owned(),
                    102,
                )
            };
            put(ui, area, 42, 2, &title, secondary);
            put(ui, area, badge_x, 2, &badge_text, badge);
            if !ink.restore_saved_layout {
                put(
                    ui,
                    area,
                    42,
                    3,
                    &"─".repeat(75),
                    plain(ink.rule.on(InkBackground::Card), ink.card),
                );
            }
            let mut y = 4u16;
            put(ui, area, 42, y, "Working dir", muted);
            put(ui, area, 56, y, frame.workdir, card);
            y = y.saturating_add(1);
            for (index, mount) in frame.mounts.iter().enumerate() {
                if index == 0 {
                    put(ui, area, 42, y, "Mounts", muted);
                }
                put(
                    ui,
                    area,
                    56,
                    y,
                    &format!("{} · {} {}", mount.source, mount.mode, mount.isolation),
                    card,
                );
                y = y.saturating_add(1);
            }
            put(ui, area, 42, y, "Roles", muted);
            put(ui, area, 56, y, frame.role_line, card);
            y = y.saturating_add(1);
            put(ui, area, 42, y, "Environments", muted);
            put(ui, area, 56, y, frame.env_line, card);
            y = y.saturating_add(1);
            for (index, line) in frame.account_lines.iter().enumerate() {
                if index == 0 {
                    put(ui, area, 42, y, "Accounts", muted);
                }
                put(ui, area, 56, y, line, card);
                y = y.saturating_add(1);
            }
            put(ui, area, 42, y, "Policies", muted);
            put(ui, area, 56, y, frame.policy_line, card);
            y = y.saturating_add(2);
            put(
                ui,
                area,
                42,
                y,
                "Instances",
                bold(ink.secondary.on(InkBackground::Card), ink.card),
            );
            let stale = frame.mode == SplitMode::Saved
                && frame.alert.is_some_and(|alert| alert.daemon_stale);
            let meta = if stale {
                format!("▲ daemon stale · {}", frame.daemon_ago)
            } else {
                format!("daemon · {}", frame.daemon_ago)
            };
            let meta_x = 117u16.saturating_sub(width(&meta));
            put(ui, area, meta_x, y, &meta, badge);
            y = y.saturating_add(1);
            for instance in frame.instances {
                let mark = if instance.live { "◉ " } else { "◌ " };
                let mark_style = if instance.live {
                    plain(ink.live.on(InkBackground::Card), ink.card)
                } else {
                    plain(ink.idle.on(InkBackground::Card), ink.card)
                };
                put(ui, area, 42, y, mark, mark_style);
                let body = format!("{}  {} · {} · ", instance.id, instance.role, instance.agent);
                if instance.live {
                    put(ui, area, 44, y, &body, card);
                    let status_x = 44u16.saturating_add(width(&body));
                    put(
                        ui,
                        area,
                        status_x,
                        y,
                        instance.status,
                        plain(ink.live.on(InkBackground::Card), ink.card),
                    );
                } else {
                    put(
                        ui,
                        area,
                        44,
                        y,
                        &format!("{body}{}", instance.status),
                        secondary,
                    );
                }
                y = y.saturating_add(1);
            }
        }
    }
}

fn paint_buttons(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let cap = ink.overlay_surface_fill;
    let cap_glyph = if ink.restore_saved_layout { "▎" } else { " " };
    let label = plain(ink.primary.on(InkBackground::Overlay), ink.button);
    match frame.mode {
        SplitMode::Empty => {
            ui.fill(Rect::new(41, 36, 8, 1), label);
            ui.fill(Rect::new(51, 36, 18, 1), label);
            put(ui, area, 41, 36, cap_glyph, cap);
            put(ui, area, 42, 36, "Launch", label);
            put(ui, area, 51, 36, cap_glyph, cap);
            put(ui, area, 52, 36, "Create workspace", label);
        }
        SplitMode::Saved | SplitMode::Expanded => {
            ui.fill(Rect::new(41, 36, 8, 1), label);
            ui.fill(Rect::new(51, 36, 6, 1), label);
            put(ui, area, 41, 36, cap_glyph, cap);
            put(ui, area, 42, 36, "Launch", label);
            put(ui, area, 51, 36, cap_glyph, cap);
            put(ui, area, 52, 36, "Edit", label);
            if frame.mode == SplitMode::Expanded {
                ui.fill(Rect::new(59, 36, 9, 1), label);
                ui.fill(Rect::new(70, 36, 9, 1), label);
                put(ui, area, 59, 36, " ", cap);
                put(ui, area, 60, 36, "Prewarm ", label);
                put(ui, area, 70, 36, " ", cap);
                put(
                    ui,
                    area,
                    71,
                    36,
                    "Delete… ",
                    label.with_fg_from(ink.danger_button.on(InkBackground::Overlay).paint_style()),
                );
            }
        }
    }
}

fn paint_hints(ui: &mut Ui<'_>, area: Rect, frame: &ManagerSplit<'_>, ink: &Ink) {
    let y = area.bottom().saturating_sub(1);
    ui.paint_style(
        Rect::new(area.x, y, area.width, 1),
        plain(ink.primary.on(InkBackground::Canvas), ink.canvas),
    );
    if frame.mode == SplitMode::Saved
        && let Some(alert) = frame.alert
    {
        paint_degraded_hints(ui, area, y, alert, ink);
        return;
    }
    let mode = frame.mode;
    let key = bold(ink.primary.on(InkBackground::Canvas), ink.canvas);
    let hint = plain(ink.muted.on(InkBackground::Canvas), ink.canvas);
    let pairs: &[(&str, &str)] = match mode {
        SplitMode::Empty => &[
            ("Enter", "Launch"),
            ("n", "New"),
            ("Tab", "Details"),
            ("c", "Accounts"),
            ("u", "Usage"),
            ("s", "Settings"),
            ("?", "Help"),
            ("q", "Quit"),
        ],
        SplitMode::Saved => &[
            ("Enter", "Launch"),
            ("n", "New"),
            ("e", "Edit"),
            ("Tab", "Details"),
            ("c", "Accounts"),
            ("u", "Usage"),
            ("s", "Settings"),
            ("?", "Help"),
            ("q", "Quit"),
        ],
        SplitMode::Expanded => &[
            ("Enter", "Launch"),
            ("→", "Expand"),
            ("e", "Edit"),
            ("n", "New"),
            ("d", "Delete…"),
            ("w", "Prewarm"),
            ("o", "GitHub"),
            ("Tab", "Details"),
            ("c", "Accounts"),
            ("u", "Usage"),
            ("?", "Help"),
        ],
    };
    let starts: &[u16] = if ink.restore_saved_layout {
        &[14, 28, 35, 43, 56, 68, 77, 89, 97]
    } else {
        match mode {
            SplitMode::Empty => &[1, 15, 22, 35, 47, 56, 68, 76],
            SplitMode::Saved => &[1, 15, 22, 30, 43, 55, 64, 76, 84],
            SplitMode::Expanded => &[1, 15, 25, 33, 40, 51, 62, 72, 85, 97, 106],
        }
    };
    for (index, (key_text, label)) in pairs.iter().enumerate() {
        let x = starts.get(index).copied().unwrap_or(1);
        put(ui, area, x, y, key_text, key);
        let label_x = x.saturating_add(width(key_text)).saturating_add(1);
        put(ui, area, label_x, y, label, hint);
    }
    if mode == SplitMode::Expanded {
        put(
            ui,
            area,
            114,
            y,
            "…",
            plain(ink.badge.on(InkBackground::Canvas), ink.canvas),
        );
    }
}

fn paint_degraded_hints(ui: &mut Ui<'_>, area: Rect, y: u16, alert: SplitAlert<'_>, ink: &Ink) {
    let key = bold(ink.primary.on(InkBackground::Canvas), ink.canvas);
    let hint = plain(ink.muted.on(InkBackground::Canvas), ink.canvas);
    put(ui, area, 1, y, "Enter", key);
    put(ui, area, 7, y, "Launch", hint);
    put(ui, area, 15, y, "?", key);
    put(ui, area, 17, y, "Help", hint);
    put(
        ui,
        area,
        23,
        y,
        "…",
        plain(ink.badge.on(InkBackground::Canvas), ink.canvas),
    );
    if alert.daemon_stale {
        put(
            ui,
            area,
            30,
            y,
            "▲",
            bold(ink.stale.on(InkBackground::Canvas), ink.canvas),
        );
    }
    if let Some(error) = alert.index_error.filter(|error| !error.is_empty()) {
        let text =
            format!(" Could not confirm running instances: {error} · list may be incomplete");
        put(
            ui,
            area,
            31,
            y,
            &text,
            plain(ink.stale.on(InkBackground::Canvas), ink.canvas),
        );
    }
}
