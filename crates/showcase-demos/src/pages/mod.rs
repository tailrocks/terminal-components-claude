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
use termrock::{id, truncate, width};

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

/// Stable identity for the page-chrome component. The shell shows one page
/// at a time, so a single id never collides across pages.
const FRAME_ID: Id = id!("showcase.page.frame");

const FRAME_FAMILY: Family = Family::custom("showcase-page-frame");
const FRAME_PARTS: &[Part] = &[Part::CONTAINER, Part::TITLE, Part::DETAIL, Part::TEXT];
const TITLE_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Primary));
const DETAIL_PATCH: StylePatch = StylePatch::new().set_fg(Role::Fg(FgStep::Muted));
const FRAME_PART_PATCHES: &[(Part, StylePatch)] =
    &[(Part::TITLE, TITLE_PATCH), (Part::DETAIL, DETAIL_PATCH)];

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
        ui.register_decor(FRAME_ID, PartRef::of(Part::TEXT), area);
        for (offset, line) in text.iter().enumerate() {
            let Ok(offset) = u16::try_from(offset) else {
                break;
            };
            if offset >= area.height {
                break;
            }
            let _ = ui.paint_str(
                Rect {
                    y: area.y.saturating_add(offset),
                    height: 1,
                    ..area
                },
                line,
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
    use super::PageFrame;
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
}
