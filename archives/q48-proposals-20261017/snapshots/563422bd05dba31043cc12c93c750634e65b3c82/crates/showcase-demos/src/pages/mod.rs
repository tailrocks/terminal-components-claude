//! The twenty-two showcase screens.
//!
//! Every screen owns the state for the controls it demonstrates. The shell
//! only selects a screen and supplies its content rectangle; this keeps the
//! application package a consumer of the public `junie-tui` facade rather than
//! a second component implementation.

use termrock::author::{
    Family, FgStep, Id, Modifier, PaintStyle, Part, PartRef, PartStyle, Rect, Response, Role,
    StateFlags, StylePatch, Ui, Variant,
};
use termrock::{id, truncate, width, wrap};

/// Product intent returned to the shell, which owns its display lifetime.
pub struct PageStatus(pub String);

/// A page preserves the component response metadata while returning any
/// status change from the same update; no later outbox drain is required.
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

/// Modal-dialog footer state: what the shell footer shows while a page
/// holds an open dialog layer (tag `app.rs` `draw_footer`). `editing`
/// selects the Enter/Esc pair; otherwise the footer shows the
/// arrow/Enter/Esc hints plus the `y / n` quick answer iff
/// `quick_answer` (a text question, never the help dialog).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModalFooter {
    /// Whether the open dialog's editor is actively editing.
    pub editing: bool,
    /// Whether the open dialog answers `y` / `n` directly.
    pub quick_answer: bool,
}

/// A stateful screen in the showcase.
pub trait Page: Send {
    /// Stable navigation title.
    fn title(&self) -> &'static str;
    /// Drain this screen's runtime intents.
    fn update(&mut self, cx: &mut termrock::Cx<'_>) -> PageUpdate;
    /// Handle an application-level command before component intents run.
    fn command(
        &mut self,
        _cx: &mut termrock::Cx<'_>,
        _action: termrock::ActionKey,
    ) -> Response<()> {
        Response::ignored()
    }
    /// Draw this screen into the shell's content rectangle.
    fn draw(&self, ui: &mut Ui<'_>, area: Rect);
    /// Contextual footer hints for the focused page or layer.
    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        &[]
    }
    /// Whether the focused page control is in edit mode.
    fn editing(&self, _ui: &Ui<'_>) -> bool {
        false
    }
    /// Footer state while this page holds an open dialog layer; `None`
    /// keeps the page hints. Only `Dialog` layers report here — menus and
    /// pickers are not dialogs.
    fn modal_footer(&self, _ui: &Ui<'_>) -> Option<ModalFooter> {
        None
    }
    /// Set an animation state for deterministic, paused inspection.
    fn seek_paused(&mut self, _frame: usize) {}
}

/// Draw a screen frame and hand its inset body to the page.
///
/// Thin composition root over [`PageFrame`]: it paints nothing itself, so
/// every page inherits the component path without per-page edits.
pub fn frame(
    ui: &mut Ui<'_>,
    area: Rect,
    title: &'static str,
    meta: &'static str,
    body: impl FnOnce(&mut Ui<'_>, Rect),
) {
    PageFrame::new(title, meta).draw(ui, area, body);
}

/// Paint a set of lines with one-cell spacing, clipping at the body edge.
///
/// Thin composition root over the [`PageFrame`] annotation path: it paints
/// nothing itself, so annotation callers inherit the component path unchanged.
pub fn lines(ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
    PageFrame::NOTES.draw_lines(ui, area, text);
}

/// Paint faint annotation lines with one-cell spacing, clipping at the body edge.
pub fn lines_faint(ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
    PageFrame::NOTES.draw_faint_lines(ui, area, text);
}

/// Paint secondary annotation lines with one-cell spacing, clipping at the body edge.
pub fn lines_secondary(ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
    PageFrame::NOTES.draw_secondary_lines(ui, area, text);
}

/// One prose line for [`lines_prose`]: body text plus whether its first
/// glyph takes the accent marker style (the tag keys that style on the
/// line's identity, so the page supplies the flag).
pub struct ProseLine<'a> {
    /// Body text, wrapped to the area width.
    pub text: &'a str,
    /// Whether the first glyph of the first wrapped row paints accent.
    pub marker: bool,
}

/// Paint wrapped prose lines, clipping at the body edge.
///
/// Thin composition root over the [`PageFrame`] prose path: it paints
/// nothing itself, so callers inherit the component path unchanged.
pub fn lines_prose(ui: &mut Ui<'_>, area: Rect, text: &[ProseLine<'_>]) {
    PageFrame::NOTES.draw_prose_lines(ui, area, text);
}

/// Stable identity for the page-chrome component. The shell shows one page
/// at a time, so a single id never collides across pages.
const FRAME_ID: Id = id!("showcase.page.frame");

const FRAME_FAMILY: Family = Family::custom("showcase-page-frame");
const FRAME_PARTS: &[Part] = &[
    Part::CONTAINER,
    Part::TITLE,
    Part::DETAIL,
    Part::TEXT,
    Part::MARKER,
];
const TITLE_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
const DETAIL_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
const TEXT_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
const FAINT_TEXT_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Faint));
const SECONDARY_TEXT_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Secondary));
const MARKER_PATCH: StylePatch = StylePatch::new().set_fg(Role::Accent);
const FRAME_PART_PATCHES: &[(Part, StylePatch)] = &[
    (Part::TITLE, TITLE_PATCH),
    (Part::DETAIL, DETAIL_PATCH),
    (Part::TEXT, TEXT_PATCH),
];
const FAINT_FRAME_PART_PATCHES: &[(Part, StylePatch)] = &[
    (Part::TITLE, TITLE_PATCH),
    (Part::DETAIL, DETAIL_PATCH),
    (Part::TEXT, FAINT_TEXT_PATCH),
];
const SECONDARY_FRAME_PART_PATCHES: &[(Part, StylePatch)] = &[
    (Part::TITLE, TITLE_PATCH),
    (Part::DETAIL, DETAIL_PATCH),
    (Part::TEXT, SECONDARY_TEXT_PATCH),
];
const PROSE_FRAME_PART_PATCHES: &[(Part, StylePatch)] = &[
    (Part::TITLE, TITLE_PATCH),
    (Part::DETAIL, DETAIL_PATCH),
    (Part::TEXT, SECONDARY_TEXT_PATCH),
    (Part::MARKER, MARKER_PATCH),
];

/// Page chrome as a downstream-authored reusable component, following the
/// [`AuthorBadge`](author::AuthorBadge) reference shape: only
/// `termrock::author` UI work plus the public [`truncate`]/[`width`]
/// geometry helpers, a declared family and parts, decor registration, and
/// theme-resolved styles.
///
/// No stock component fits this chrome pixel-exactly: `Panel` adds a focus
/// gutter, card insets and a raised surface, `TextViewport` adds a
/// scrollbar and focus registration, and `StatusBar` adds item separators.
/// Per ARC-008 the author extension is therefore the correct contract
/// level; per ARC-003 the ellipsis geometry is delegated to the owned
/// [`truncate`] primitive rather than hand-rolled here.
///
/// The component is stateless chrome: it has no `update`, registers decor
/// (never a focus stop), and resolves styles with empty flags, matching the
/// legacy state-independent output exactly.
pub struct PageFrame {
    title: &'static str,
    meta: &'static str,
}

impl PageFrame {
    pub(crate) const fn new(title: &'static str, meta: &'static str) -> Self {
        Self { title, meta }
    }

    /// Shared renderer for annotation lines; `draw_lines` uses no
    /// title/meta, so one instance serves every caller.
    pub(crate) const NOTES: PageFrame = PageFrame::new("", "");

    /// The declared part contract with its instance patches: `TITLE`
    /// carries primary foreground (post-resolution `BOLD` is the correct
    /// equivalence choice because it matches the retired layering
    /// exactly), `DETAIL` carries muted foreground, and
    /// `CONTAINER`/`TEXT` resolve to the neutral surface.
    fn styles() -> PartStyle<'static> {
        PartStyle::new()
            .declare(FRAME_PARTS)
            .part(FRAME_PART_PATCHES)
    }

    fn faint_styles() -> PartStyle<'static> {
        PartStyle::new()
            .declare(FRAME_PARTS)
            .part(FAINT_FRAME_PART_PATCHES)
    }

    fn secondary_styles() -> PartStyle<'static> {
        PartStyle::new()
            .declare(FRAME_PARTS)
            .part(SECONDARY_FRAME_PART_PATCHES)
    }

    fn prose_styles() -> PartStyle<'static> {
        PartStyle::new()
            .declare(FRAME_PARTS)
            .part(PROSE_FRAME_PART_PATCHES)
    }

    /// Resolve one part layered over the current surface (§11.3 final
    /// layering, as `Panel` does): unpatched slots inherit the surface, so
    /// `TITLE`/`DETAIL`/`TEXT` bind exactly the legacy styles. Empty flags:
    /// the chrome carries no interactive state.
    fn part_style(ui: &mut Ui<'_>, part: Part) -> PaintStyle {
        let base = ui.surface_style();
        Self::styles()
            .style(
                ui,
                FRAME_ID,
                FRAME_FAMILY,
                Variant::DEFAULT,
                part,
                StateFlags::empty(),
            )
            .over(base)
    }

    fn faint_part_style(ui: &mut Ui<'_>, part: Part) -> PaintStyle {
        let base = ui.surface_style();
        Self::faint_styles()
            .style(
                ui,
                FRAME_ID,
                FRAME_FAMILY,
                Variant::DEFAULT,
                part,
                StateFlags::empty(),
            )
            .over(base)
    }

    fn secondary_part_style(ui: &mut Ui<'_>, part: Part) -> PaintStyle {
        let base = ui.surface_style();
        Self::secondary_styles()
            .style(
                ui,
                FRAME_ID,
                FRAME_FAMILY,
                Variant::DEFAULT,
                part,
                StateFlags::empty(),
            )
            .over(base)
    }

    fn prose_part_style(ui: &mut Ui<'_>, part: Part) -> PaintStyle {
        let base = ui.surface_style();
        Self::prose_styles()
            .style(
                ui,
                FRAME_ID,
                FRAME_FAMILY,
                Variant::DEFAULT,
                part,
                StateFlags::empty(),
            )
            .over(base)
    }

    /// Paint the title row and hand the inset body to the page.
    ///
    /// Geometry and paint order match the historical helper exactly: title
    /// row, blank row, then page content, with no second card around the
    /// page. An empty area runs no body (unlike `Panel::draw`, which always
    /// runs its body once); a short area still runs the body on the
    /// saturated empty rect.
    pub(crate) fn draw(&self, ui: &mut Ui<'_>, area: Rect, body: impl FnOnce(&mut Ui<'_>, Rect)) {
        if area.is_empty() {
            return;
        }
        ui.register_decor(FRAME_ID, PartRef::of(Part::CONTAINER), area);
        let container = Self::part_style(ui, Part::CONTAINER);
        ui.fill(area, container);
        let title_area = Rect { height: 1, ..area };
        ui.register_decor(FRAME_ID, PartRef::of(Part::TITLE), title_area);
        let title_width = width(self.title).min(area.width);
        let title = Self::part_style(ui, Part::TITLE).add_modifier(Modifier::BOLD);
        ui.paint_str(title_area, self.title, title);
        if !self.meta.is_empty() && area.width > title_width.saturating_add(4) {
            self.paint_meta(
                ui,
                Rect {
                    x: area.x.saturating_add(title_width).saturating_add(2),
                    width: area.width.saturating_sub(title_width).saturating_sub(3),
                    height: 1,
                    ..area
                },
            );
        }
        body(
            ui,
            Rect {
                y: area.y.saturating_add(2),
                height: area.height.saturating_sub(2),
                ..area
            },
        );
    }

    /// Paint the meta run with end-ellipsis clipping. Truncation geometry
    /// is owned by [`truncate`]; the component only supplies the styled run
    /// through the clip-honoring [`Ui::paint_str`] primitive.
    fn paint_meta(&self, ui: &mut Ui<'_>, meta_area: Rect) {
        if meta_area.is_empty() {
            return;
        }
        ui.register_decor(FRAME_ID, PartRef::of(Part::DETAIL), meta_area);
        let clipped = truncate(self.meta, meta_area.width);
        let detail = Self::part_style(ui, Part::DETAIL);
        ui.paint_str(meta_area, &clipped, detail);
    }

    /// Paint annotation lines with one-cell spacing, clipping at the edge.
    pub(crate) fn draw_lines(&self, ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
        let style = Self::part_style(ui, Part::TEXT);
        self.draw_styled_lines(ui, area, text, style);
    }

    /// Paint faint annotation lines with one-cell spacing, clipping at the edge.
    pub(crate) fn draw_faint_lines(&self, ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
        let style = Self::faint_part_style(ui, Part::TEXT);
        self.draw_styled_lines(ui, area, text, style);
    }

    /// Paint secondary annotation lines with one-cell spacing, clipping at the edge.
    pub(crate) fn draw_secondary_lines(&self, ui: &mut Ui<'_>, area: Rect, text: &[&str]) {
        let style = Self::secondary_part_style(ui, Part::TEXT);
        self.draw_styled_lines(ui, area, text, style);
    }

    /// Paint prose lines wrapped to the area width, clipping at the edge.
    ///
    /// Wrap geometry is owned by the shared [`wrap`] primitive (the same
    /// word/hard-wrap walk the tag's `text::wrap` runs); rows flow
    /// top-down across lines and stop at the area bottom. Body rows take
    /// secondary; a marked line's first wrapped row paints its first
    /// character accent (`tag:sidebars.rs:431-450`).
    pub(crate) fn draw_prose_lines(&self, ui: &mut Ui<'_>, area: Rect, text: &[ProseLine<'_>]) {
        ui.register_decor(FRAME_ID, PartRef::of(Part::TEXT), area);
        let body = Self::prose_part_style(ui, Part::TEXT);
        let marker = Self::prose_part_style(ui, Part::MARKER);
        let rows = text.iter().flat_map(|line| {
            wrap(line.text, area.width)
                .into_iter()
                .enumerate()
                .map(|(index, wrapped)| (wrapped, line.marker && index == 0))
        });
        let mut row = 0u16;
        for (wrapped, marked) in rows.take(usize::from(area.height)) {
            let row_area = Rect {
                y: area.y.saturating_add(row),
                height: 1,
                ..area
            };
            if marked {
                self.paint_marker_row(ui, row_area, &wrapped, marker, body);
            } else {
                self.paint_row(ui, row_area, &wrapped, body);
            }
            row = row.saturating_add(1);
        }
    }

    /// Paint one marker row: the first character accent, the rest body.
    ///
    /// An empty row (a wrapped blank line) paints nothing; the glyph run
    /// only ever covers its own measured width, mirroring the tag's two
    /// `set_string` runs.
    fn paint_marker_row(
        &self,
        ui: &mut Ui<'_>,
        row_area: Rect,
        wrapped: &str,
        marker: PaintStyle,
        body: PaintStyle,
    ) {
        let Some(first) = wrapped.chars().next() else {
            self.paint_row(ui, row_area, wrapped, body);
            return;
        };
        let glyph = &wrapped[..first.len_utf8()];
        let glyph_width = width(glyph).min(row_area.width);
        let glyph_area = Rect {
            width: glyph_width,
            ..row_area
        };
        ui.register_decor(FRAME_ID, PartRef::of(Part::MARKER), glyph_area);
        self.paint_row(ui, glyph_area, glyph, marker);
        let rest_area = Rect {
            x: row_area.x.saturating_add(glyph_width),
            width: row_area.width.saturating_sub(glyph_width),
            ..row_area
        };
        self.paint_row(ui, rest_area, &wrapped[first.len_utf8()..], body);
    }

    /// Paint one pre-clipped annotation row: the single row-painting site
    /// shared by the truncated and wrapped annotation paths.
    fn paint_row(&self, ui: &mut Ui<'_>, row_area: Rect, text: &str, style: PaintStyle) {
        let _ = ui.paint_str(row_area, text, style);
    }

    fn draw_styled_lines(&self, ui: &mut Ui<'_>, area: Rect, text: &[&str], style: PaintStyle) {
        ui.register_decor(FRAME_ID, PartRef::of(Part::TEXT), area);
        for (offset, line) in text.iter().enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            if offset >= area.height {
                break;
            }
            let text_to_paint = truncate(line, area.width);
            self.paint_row(
                ui,
                Rect {
                    y: area.y.saturating_add(offset),
                    height: 1,
                    ..area
                },
                &text_to_paint,
                style,
            );
        }
    }
}

pub mod author;
pub mod buttons;
pub mod chips;
pub mod chrome;
pub mod dialogs;
pub mod diff;
pub mod editable;
pub mod editor;
pub mod forms;
pub mod grid;
pub mod inputs;
pub mod lists;
pub mod overview;
pub mod panels;
pub mod pickers;
pub mod progress;
pub mod scrolling;
pub mod settings;
pub mod sidebars;
pub mod tables;
pub mod taskrunner;
pub mod terminal;
pub mod textareas;
pub mod trees;

#[cfg(test)]
mod clipping_tests {
    use super::{PageFrame, ProseLine, lines_prose};
    use termrock::{App, Cx, FgStep, Rect, Response, Theme, Ui};
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
            // Drive the component's meta path (DETAIL resolution plus the
            // owned truncation geometry), not the retired free function.
            PageFrame::new("", self.text).paint_meta(ui, Rect::new(0, 0, self.columns, 1));
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
                // The component resolves DETAIL to muted, matching the
                // production meta style the retired helper computed.
                assert_eq!(
                    Some(h.cell(0, 0).fg),
                    Theme::junie().color.fg.get(FgStep::Muted.index()).copied()
                );
            }
        }
    }

    struct ProseSample;
    impl App for ProseSample {
        fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            lines_prose(
                ui,
                Rect::new(0, 0, 8, 4),
                &[
                    ProseLine {
                        text: "aa bb cc dd",
                        marker: false,
                    },
                    ProseLine {
                        text: "›  current item",
                        marker: true,
                    },
                ],
            );
        }
    }

    #[test]
    fn q67b2_prose_wraps_and_marks_first_glyph() {
        let h = Harness::new(ProseSample, Theme::junie(), 8, 4);
        // Shared wrap geometry: "aa bb cc dd" breaks after "cc",
        // "›  current item" breaks after the leading "› ".
        assert_eq!(h.row(0), "aa bb cc");
        assert_eq!(h.row(1), "dd      ");
        assert_eq!(h.row(2), "›       ");
        assert_eq!(h.row(3), "current ");
        let theme = Theme::junie();
        let secondary = theme.color.fg.get(FgStep::Secondary.index()).copied();
        assert_eq!(Some(h.cell(0, 0).fg), secondary);
        assert_eq!(Some(h.cell(0, 3).fg), secondary);
        // Marked line: first glyph accent, the rest secondary.
        assert_eq!(h.cell(0, 2).symbol(), "›");
        assert_eq!(h.cell(0, 2).fg, theme.color.accent);
        assert_eq!(Some(h.cell(1, 2).fg), secondary);
    }
}
