use termrock::{
    ActionKey, Color, Family, FgStep, FrameRead, Id, Modifier, Panel, PanelKind, PanelWindowError,
    Part, Rect, Response, Role, StateFlags, StylePatch, Surface, Ui, Variant,
    author::VerticalWindow, width,
};

pub struct PageStatus(pub String);

pub struct PageUpdate {
    pub response: Response<()>,
    pub status: Option<PageStatus>,
}

impl From<Response<()>> for PageUpdate {
    fn from(response: Response<()>) -> Self {
        Self {
            response,
            status: None,
        }
    }
}

pub trait Page: Send {
    fn title(&self) -> &'static str;
    fn update(&mut self, cx: &mut termrock::Cx<'_>) -> PageUpdate;
    fn option_value(&self, _key: &str) -> Option<usize> {
        None
    }
    fn set_option(&mut self, _key: &str, _value: usize) {}
    /// Native focus owners that may receive an application-level captured action.
    ///
    /// The app checks the returned owners against the live focus ring and top layer both
    /// before installing the capture binding and again when dispatching the action.
    fn global_capture_owners(&self, _action: ActionKey) -> &'static [Id] {
        &[]
    }
    fn command(
        &mut self,
        _cx: &mut termrock::Cx<'_>,
        _action: termrock::ActionKey,
    ) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[]
    }
    fn editing(&self, _ui: &Ui<'_>) -> bool {
        false
    }
    fn seek_paused(&mut self, _frame: usize) {}
}

/// The selection fill behind chosen rows. Works around the junie theme
/// computing `accent_tint` to `#000f2e13` where the reference uses
/// `#173f1c`; other themes pass through untouched.
pub fn accent_tint_bg(theme: &termrock::Theme) -> u32 {
    let drifted = termrock::Color::from_u32(0x000f_2e13);
    if theme.color.accent_tint == drifted {
        0x0017_3f1c
    } else {
        color_u32(theme.color.accent_tint)
    }
}

/// Packs an [`termrock::Color`] into the `0x00rrggbb` form the page paint
/// helpers use. Non-RGB colours degrade to their index (or black).
pub fn color_u32(color: termrock::Color) -> u32 {
    match color {
        termrock::Color::Rgb(r, g, b) => (u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b),
        termrock::Color::Indexed(i) => u32::from(i),
        _ => 0,
    }
}

fn heading_style(ui: &mut Ui<'_>, family: Family, part: Part) -> termrock::author::PaintStyle {
    let surface = ui.surface_style();
    let resolved = ui.style(family, Variant::DEFAULT, part, StateFlags::empty());
    surface.patch(resolved.style)
}

pub fn frame(
    ui: &mut Ui<'_>,
    area: Rect,
    title: &'static str,
    meta: &'static str,
    body: impl FnOnce(&mut Ui<'_>, Rect),
) {
    if area.is_empty() {
        return;
    }
    ui.fill(area, ui.surface_style());
    let title_style = heading_style(ui, Family::PANEL, Part::TITLE);
    let meta_style = heading_style(ui, Family::LIST, Part::META);
    let title_area = Rect { height: 1, ..area };
    let title_width = width(title).min(area.width);
    ui.paint_str(title_area, title, title_style);
    if !meta.is_empty() && area.width > title_width.saturating_add(4) {
        let meta_area = Rect {
            x: area.x.saturating_add(title_width).saturating_add(2),
            width: area.width.saturating_sub(title_width).saturating_sub(3),
            height: 1,
            ..area
        };
        paint_clipped_meta(ui, meta_area, meta, meta_style);
    }
    let body_area = Rect {
        y: area.y.saturating_add(2),
        height: area.height.saturating_sub(2),
        ..area
    };
    body(ui, body_area);
}

fn paint_clipped_meta(
    ui: &mut Ui<'_>,
    meta_area: Rect,
    meta: &str,
    meta_style: termrock::author::PaintStyle,
) {
    if meta_area.is_empty() {
        return;
    }
    if width(meta) <= meta_area.width {
        ui.paint_str(meta_area, meta, meta_style);
    } else {
        let budget = Rect {
            width: meta_area.width.saturating_sub(1),
            ..meta_area
        };
        let used = ui.paint_str(budget, meta, meta_style);
        ui.paint_str(
            Rect::new(meta_area.x.saturating_add(used), meta_area.y, 1, 1),
            "…",
            meta_style,
        );
    }
}

/// Preserves the old one-column compact-page metadata admission at the exact
/// title-plus-four boundary while keeping the shared semantic metadata style.
/// The general [`frame`] gate intentionally remains stricter for every other
/// page.
pub(super) fn paint_boundary_meta(ui: &mut Ui<'_>, area: Rect, title: &str, meta: &str) {
    let title_width = width(title);
    if area.is_empty() || meta.is_empty() || area.width != title_width.saturating_add(4) {
        return;
    }
    let meta_area = Rect {
        x: area.x.saturating_add(title_width).saturating_add(2),
        width: 1,
        height: 1,
        ..area
    };
    let style = heading_style(ui, Family::LIST, Part::META);
    paint_clipped_meta(ui, meta_area, meta, style);
}

/// The frozen capsule seam ink.
///
/// The theme resolves the library's tab rule to `border_subtle` (`#262626`),
/// while the approved frames paint the capsule seam `#3a3a3a`. The theme
/// exposes no public token for that ink, so the demo layer names it once,
/// here. Library need: a public seam token on `Theme`.
const SEAM: u32 = 0x003a_3a3a;

/// The panel title ink the playground cards use: secondary, never bold, with
/// the meta one step fainter.
const CARD_TITLE: [(Part, StylePatch); 2] = [
    (
        Part::TITLE,
        StylePatch::new()
            .set_fg(Role::Fg(FgStep::Secondary))
            .remove(Modifier::BOLD),
    ),
    (
        Part::DETAIL,
        StylePatch::new().set_fg(Role::Fg(FgStep::Faint)),
    ),
];

/// The section strip every playground page opens with: the section tabs,
/// their rule and the caption that names the live section.
pub fn section_strip(ui: &mut Ui<'_>, area: Rect) {
    const SECTIONS: [&str; 3] = ["Playground", "Options", "States"];
    const CAPTION: &str = "the live widget";
    const WIDE: u16 = 76;
    if area.is_empty() {
        return;
    }
    let canvas = ui.surface_style();
    let active_tab = canvas.patch(
        ui.paint_patch(
            &StylePatch::new()
                .set_fg(Role::Fg(FgStep::Primary))
                .set_bg(Role::Surface(Surface::Overlay))
                .add(Modifier::BOLD),
        ),
    );
    let idle_tab =
        canvas.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary))));
    let caption_style =
        canvas.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));
    let seam = canvas
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::from_u32(SEAM)))));
    let accent_seam = canvas.patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Accent)));
    let tab_width = |label: &str| width(label).saturating_add(3);
    let mut x = area.x;
    let mut tab_end = area.x;
    for (index, label) in SECTIONS.iter().enumerate() {
        let w = tab_width(label);
        if x.saturating_add(w) > area.right() {
            break;
        }
        let style = if index == 0 { active_tab } else { idle_tab };
        ui.fill(Rect::new(x, area.y, w, 1), style);
        ui.paint_str(
            Rect::new(x.saturating_add(1), area.y, w.saturating_sub(1), 1),
            label,
            style,
        );
        tab_end = x.saturating_add(w);
        x = tab_end.saturating_add(1);
    }
    if area.width >= WIDE {
        let caption_width = width(CAPTION);
        let caption_x = area.right().saturating_sub(caption_width.saturating_add(1));
        if caption_x > tab_end.saturating_add(2) {
            ui.paint_str(
                Rect::new(caption_x, area.y, caption_width, 1),
                CAPTION,
                caption_style,
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
        seam,
    );
    let active = tab_width(SECTIONS[0]).min(area.width);
    ui.paint_str(
        Rect::new(area.x, rule_y, active, 1),
        &"━".repeat(usize::from(active)),
        accent_seam,
    );
}

/// The card panel the playground pages draw: the panel's own title row, the
/// frozen seam under it and the body below.
///
/// The panel widget owns the title, the meta and the body; the demo layer
/// owns the seam the approved frames paint under the title.
pub fn card_panel(
    ui: &mut Ui<'_>,
    id: Id,
    title: &str,
    meta: Option<&str>,
    area: Rect,
    body: impl FnOnce(&mut Ui<'_>, Rect),
) {
    let mut panel = Panel::new(id).kind(PanelKind::Card).title(title);
    if let Some(meta) = meta {
        panel = panel.meta(meta);
    }
    let inner = panel.inner(ui, area);
    panel
        .patch_part(&CARD_TITLE)
        .draw(ui, area, |ui, inner| body(ui, inner));
    if area.height < 2 || inner.is_empty() {
        return;
    }
    // The seam sits between the panel's title row and its body, on the
    // panel's own plane.
    let plane = ui.theme().bg(ui.theme().raise(ui.surface()));
    let seam = ui
        .surface_style()
        .bg(plane)
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::from_u32(SEAM)))));
    ui.paint_str(
        Rect {
            x: inner.x,
            y: area.y.saturating_add(1),
            width: inner.width,
            height: 1,
        },
        &"─".repeat(usize::from(inner.width)),
        seam,
    );
}

/// Draw a card panel through a validated vertical window into its logical rows.
///
/// The callback receives the visible portion of the panel's original inner
/// body. The title and capsule seam are painted only when their source rows are
/// visible, so scrolling does not move later body rows into their place.
pub fn card_panel_window(
    ui: &mut Ui<'_>,
    id: Id,
    title: &str,
    meta: Option<&str>,
    logical_area: Rect,
    window: VerticalWindow,
    body: impl FnOnce(&mut Ui<'_>, Option<VerticalWindow>),
) -> Result<(), PanelWindowError> {
    let mut panel = Panel::new(id).kind(PanelKind::Card).title(title);
    if let Some(meta) = meta {
        panel = panel.meta(meta);
    }
    let inner = panel.inner(ui, logical_area);
    panel
        .patch_part(&CARD_TITLE)
        .draw_window(ui, logical_area, window, body)?;
    if logical_area.height < 2 || inner.is_empty() {
        return Ok(());
    }
    let Some(y) = window.physical_row_for_source(1) else {
        return Ok(());
    };
    let plane = ui.theme().bg(ui.theme().raise(ui.surface()));
    let seam = ui
        .surface_style()
        .bg(plane)
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Custom(Color::from_u32(SEAM)))));
    ui.paint_str(
        Rect {
            x: inner.x,
            y,
            width: inner.width,
            height: 1,
        },
        &"─".repeat(usize::from(inner.width)),
        seam,
    );
    Ok(())
}

pub fn lines(ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
    let style = ui.surface_style();
    for (offset, line) in text.iter().enumerate() {
        let Ok(offset) = u16::try_from(offset) else {
            break;
        };
        if offset >= area.height {
            break;
        }
        let row = Rect {
            y: area.y.saturating_add(offset),
            height: 1,
            ..area
        };
        let _ = ui.paint_str(row, line, style);
    }
}

pub mod author;
pub mod buttons;
pub mod celleditors;
pub mod checks_radios;
pub mod chips;
pub mod chrome;
pub mod dialog_kinds;
pub mod dialogs;
pub mod diff;
pub mod editable;
pub mod editor;
pub mod empty_error;
pub mod forms;
pub mod grid;
pub mod inputs;
pub mod lists;
pub mod master_detail;
pub mod notices;
pub mod overview;
pub mod panels;
pub mod pickers;
pub mod progress;
pub mod query_workbench;
pub mod scrolling;
pub mod selectable;
pub mod settings;
pub mod sidebars;
pub mod tables;
pub mod taskrunner;
pub mod terminal;
pub mod textareas;
pub mod toolbars;
pub mod trees;

#[cfg(test)]
mod clipping_tests {
    use super::paint_clipped_meta;
    use termrock::{App, Cx, FgStep, Rect, Response, Role, StylePatch, Theme, Ui};
    use termrock_test_support::Harness;

    struct Sample {
        columns: u16,
        text: &'static str,
    }
    impl App for Sample {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let style = ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)));
            paint_clipped_meta(ui, Rect::new(0, 0, self.columns, 1), self.text, style);
        }
    }

    #[test]
    fn clipped_meta_preserves_combining_and_wide_boundaries() {
        for (columns, text, expected) in [
            (0, "e\u{301}中x", ""),
            (1, "e\u{301}中x", "…"),
            (2, "e\u{301}中x", "e\u{301}…"),
            (3, "e\u{301}中x", "e\u{301}…"),
            (4, "e\u{301}中x", "e\u{301}中x"),
            (1, "中a", "…"),
            (2, "中a", "…"),
            (3, "中a", "中a"),
        ] {
            let h = Harness::new(Sample { columns, text }, Theme::junie(), 8, 1);
            assert_eq!(h.text().trim_end(), expected, "columns={columns}");
            assert_eq!(
                h.cell(columns, 0).symbol(),
                " ",
                "clip must not escape its area"
            );
            if columns > 0 {
                assert_eq!(
                    Some(h.cell(0, 0).fg),
                    Theme::junie()
                        .color
                        .fg
                        .get(FgStep::Secondary.index())
                        .copied()
                );
            }
        }
    }
}
