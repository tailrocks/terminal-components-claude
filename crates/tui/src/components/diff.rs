//! `DiffView` — unified and side-by-side review over `TextViewport`.

use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::Modifier;

use super::SlotFn;
use super::viewport::{ProjectedText, TextViewport, ViewportAction, ViewportCmd, ViewportState};
use crate::id::{Id, Part};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::Response;
use crate::text::truncate;
use crate::theme::{FgStep, Role, StylePatch};
use crate::ui::{Cx, FrameRead, Ui};

/// Kind of one source line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffLineKind {
    /// Present on both sides.
    Context,
    /// Present only on the new side.
    Add,
    /// Present only on the old side.
    Remove,
}

/// One row exposed by a [`DiffSource`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffRow<'a> {
    /// Start of a hunk.
    Hunk {
        /// First old-side line number.
        old_start: usize,
        /// First new-side line number.
        new_start: usize,
    },
    /// A text line.
    Line {
        /// Its diff kind.
        kind: DiffLineKind,
        /// Text without a diff marker.
        text: &'a str,
    },
}

/// Borrowed source for one changed file.
///
/// This indexed projection lets an application expose its own change model
/// without converting or cloning it. [`DiffSource::revision`] must change
/// whenever any returned value changes.
pub trait DiffSource {
    /// Stable revision of every value exposed by this source.
    fn revision(&self) -> u64;
    /// Current path.
    fn path(&self) -> &str;
    /// One-letter status marker (`A`, `M`, `D`, or `R`).
    fn status_marker(&self) -> &str;
    /// Human-readable status.
    fn status_label(&self) -> &str;
    /// Semantic marker role.
    fn status_role(&self) -> Role {
        Role::Fg(FgStep::Secondary)
    }
    /// Previous path for a rename.
    fn renamed_from(&self) -> Option<&str> {
        None
    }
    /// Number of [`DiffRow`] values.
    fn row_count(&self) -> usize;
    /// Row at `index`.
    fn row(&self, index: usize) -> Option<DiffRow<'_>>;
}

/// Diff presentation.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DiffMode {
    /// Classic unified listing.
    #[default]
    Unified,
    /// Old and new sides in parallel columns.
    Review,
}

impl DiffMode {
    /// User-facing label.
    pub const fn label(self) -> &'static str {
        match self {
            DiffMode::Unified => "unified",
            DiffMode::Review => "review",
        }
    }

    /// The other presentation.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            DiffMode::Unified => DiffMode::Review,
            DiffMode::Review => DiffMode::Unified,
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct OwnedRun {
    text: String,
    role: Option<Role>,
    modifier: Modifier,
}

impl OwnedRun {
    fn plain(text: impl Into<String>) -> Self {
        OwnedRun {
            text: text.into(),
            role: None,
            modifier: Modifier::empty(),
        }
    }

    fn role(text: impl Into<String>, role: Role) -> Self {
        OwnedRun {
            text: text.into(),
            role: Some(role),
            modifier: Modifier::empty(),
        }
    }

    fn bold(mut self) -> Self {
        self.modifier |= Modifier::BOLD;
        self
    }
}

type OwnedLine = Vec<OwnedRun>;

/// Durable interaction state of a [`DiffView`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DiffViewState {
    viewport: ViewportState,
    mode: DiffMode,
}

impl Default for DiffViewState {
    fn default() -> Self {
        DiffViewState {
            viewport: ViewportState::default(),
            mode: DiffMode::Unified,
        }
    }
}

impl DiffViewState {
    /// Current mode.
    pub const fn mode(&self) -> DiffMode {
        self.mode
    }

    /// Select a mode. The next update rebuilds the projection.
    pub fn set_mode(&mut self, mode: DiffMode) {
        if self.mode != mode {
            self.mode = mode;
        }
    }

    /// Toggle and return the mode.
    pub fn toggle_mode(&mut self) -> DiffMode {
        self.set_mode(self.mode.toggled());
        self.mode
    }

    /// Embedded viewport state.
    pub const fn viewport(&self) -> &ViewportState {
        &self.viewport
    }

    /// Embedded viewport state, mutably.
    pub const fn viewport_mut(&mut self) -> &mut ViewportState {
        &mut self.viewport
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
struct DiffLayout {
    mode: DiffMode,
    revision: u64,
    width: u16,
    has_source: bool,
    valid: bool,
    scratch: Vec<OwnedLine>,
    projection: ProjectedText,
}

impl DiffLayout {
    fn ensure(&mut self, source: Option<&dyn DiffSource>, mode: DiffMode, width: u16) -> bool {
        let revision = source.map_or(0, DiffSource::revision);
        let has_source = source.is_some();
        if revision != u64::MAX
            && self.valid
            && self.revision == revision
            && self.mode == mode
            && self.width == width
            && self.has_source == has_source
        {
            return false;
        }
        self.scratch.clear();
        match source {
            Some(source) => match mode {
                DiffMode::Unified => unified_lines(source, &mut self.scratch),
                DiffMode::Review => review_lines(source, width, &mut self.scratch),
            },
            None => self.scratch.push(vec![OwnedRun::role(
                "No file selected",
                Role::Fg(FgStep::Muted),
            )]),
        }
        self.projection.clear();
        for line in &self.scratch {
            self.projection
                .push_line(line.iter().map(|run| (&run.text, run.role, run.modifier)));
        }
        self.revision = revision;
        self.mode = mode;
        self.width = width;
        self.has_source = has_source;
        self.valid = true;
        true
    }
}

/// A borrowed diff projection composed over [`TextViewport`].
///
/// ## Construction
/// `DiffView::new(id, source)`; pass `None` for the no-file state.
///
/// ## Ownership
/// The caller owns the [`DiffSource`] and [`DiffViewState`]. The runtime owns
/// focus, hit routing and the viewport layout cache.
///
/// ## Configuration
/// `.patch`, `.patch_part`, `.slot`; mode belongs to state.
///
/// ## Variants
/// `Family::DIFF`, `Variant::DEFAULT`; presentation uses [`DiffMode`].
///
/// ## States
/// Inherits viewport focus, hover, press and selection states.
///
/// ## Actions
/// [`ViewportAction`] is forwarded unchanged: copy, selection and follow.
///
/// ## Focus
/// Exactly the embedded viewport's one focus stop; no wrapper stop.
///
/// ## Keyboard
/// Viewport bindings (`↑/↓`, pages, home/end, follow and copy). Mode switching
/// is an application command calling [`DiffViewState::toggle_mode`].
///
/// ## Mouse
/// Selection drag, double-click, wheel and scrollbar are the viewport's.
///
/// ## Layout
/// Review rows measure equal columns around ` │ ` during update using the
/// last-frame width. `measure` prefers 80×12; zero area remains inert.
///
/// ## Parts
/// Exactly [`TextViewport::PARTS`]: `CONTAINER`, `TEXT`, `GUTTER`, `TRACK`,
/// `THUMB`.
///
/// ## Overrides
/// Every channel forwards to the viewport; its slot restrictions still apply.
///
/// ## Identity
/// One `Id`; source identity is the caller-owned [`DiffSource::revision`].
///
/// ## Testing
/// `DiffViewCase`; `render::components::diff_view::*`; unit tests retain the
/// legacy unified/review/count/scroll behavior through a trait adapter.
///
/// ## Invariants
/// No parallel diff data model and no render-time state transition. Rebuilds
/// occur only for source revision, mode, or measured width changes.
pub struct DiffView<'a> {
    id: Id,
    source: Option<&'a dyn DiffSource>,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    slot: Option<(Part, SlotFn<'a>)>,
}

impl fmt::Debug for DiffView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiffView")
            .field("id", &self.id)
            .field("source", &self.source.is_some())
            .field("parts", &self.parts.len())
            .field("slot", &self.slot.map(|(part, _)| part))
            .finish_non_exhaustive()
    }
}

impl<'a> DiffView<'a> {
    /// Styled parts, exactly the embedded viewport's parts.
    pub const PARTS: &'static [Part] = TextViewport::PARTS;

    /// A diff view over `source`.
    pub const fn new(id: Id, source: Option<&'a dyn DiffSource>) -> Self {
        DiffView {
            id,
            source,
            patch: None,
            parts: &[],
            slot: None,
        }
    }

    /// Id.
    pub const fn id(&self) -> Id {
        self.id
    }

    /// Patch every viewport part.
    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    /// Patch viewport parts.
    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self
    }

    /// Replace one viewport slot.
    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.slot = Some((part, slot));
        self
    }

    fn viewport(&self) -> TextViewport<'a> {
        let mut viewport = TextViewport::new(self.id).patch_part(self.parts);
        if let Some(patch) = self.patch {
            viewport = viewport.patch(patch);
        }
        if let Some((part, slot)) = self.slot {
            viewport = viewport.slot(part, slot);
        }
        viewport
    }

    /// Rebuild when needed, then delegate interaction to `TextViewport`.
    pub fn update(&self, cx: &mut Cx<'_>, state: &mut DiffViewState) -> Response<ViewportAction> {
        let width = cx.area(self.id).map_or(80, |area| area.width);
        let mut layout = core::mem::take(cx.cache::<DiffLayout>(self.id));
        if layout.ensure(self.source, state.mode, width) {
            state.viewport.set_follow(false);
            state.viewport.invalidate();
            let _ = state.viewport.clear_selection();
        }
        let response =
            self.viewport()
                .update_projected(cx, &mut state.viewport, &layout.projection);
        *cx.cache::<DiffLayout>(self.id) = layout;
        response
    }

    /// Draw the measured projection through `TextViewport`.
    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &DiffViewState) -> Rect {
        let mut layout = core::mem::take(ui.cache::<DiffLayout>(self.id));
        layout.ensure(self.source, state.mode, area.width);
        let used = self
            .viewport()
            .draw_projected(ui, area, &state.viewport, &layout.projection);
        *ui.cache::<DiffLayout>(self.id) = layout;
        used
    }

    /// Preferred review surface.
    pub fn measure(&self, _ui: &Ui<'_>, constraints: Constraints) -> Size {
        Size {
            min: (24, 3),
            preferred: (80, 12),
        }
        .fit(constraints)
    }
}

impl Bindings for DiffView<'_> {
    type Cmd = ViewportCmd;

    fn bindings(&self, state: BindingState) -> &'static [Binding<ViewportCmd>] {
        self.viewport().bindings(state)
    }
}

fn source_rows(source: &dyn DiffSource) -> impl Iterator<Item = DiffRow<'_>> {
    (0..source.row_count()).filter_map(|index| source.row(index))
}

fn counts(source: &dyn DiffSource) -> (usize, usize, usize) {
    let mut additions = 0usize;
    let mut deletions = 0usize;
    let mut hunks = 0usize;
    for row in source_rows(source) {
        match row {
            DiffRow::Hunk { .. } => hunks = hunks.saturating_add(1),
            DiffRow::Line {
                kind: DiffLineKind::Add,
                ..
            } => additions = additions.saturating_add(1),
            DiffRow::Line {
                kind: DiffLineKind::Remove,
                ..
            } => deletions = deletions.saturating_add(1),
            DiffRow::Line {
                kind: DiffLineKind::Context,
                ..
            } => {}
        }
    }
    (additions, deletions, hunks)
}

fn header(source: &dyn DiffSource, mode: DiffMode) -> OwnedLine {
    let (adds, removes, hunks) = counts(source);
    let path = source.renamed_from().map_or_else(
        || source.path().to_owned(),
        |from| format!("{from} → {}", source.path()),
    );
    let suffix = if mode == DiffMode::Review {
        " · review"
    } else {
        ""
    };
    vec![
        OwnedRun::role(source.status_marker(), source.status_role()).bold(),
        OwnedRun::plain(" "),
        OwnedRun::plain(path).bold(),
        OwnedRun::role(
            format!(
                "  +{adds} −{removes} · {hunks} hunk{}{suffix}",
                if hunks == 1 { "" } else { "s" }
            ),
            Role::Fg(FgStep::Muted),
        ),
    ]
}

fn hunk_lengths(source: &dyn DiffSource, start: usize) -> (usize, usize) {
    let mut old = 0usize;
    let mut new = 0usize;
    for index in start.saturating_add(1)..source.row_count() {
        match source.row(index) {
            Some(DiffRow::Hunk { .. }) | None => break,
            Some(DiffRow::Line { kind, .. }) => {
                if kind != DiffLineKind::Add {
                    old = old.saturating_add(1);
                }
                if kind != DiffLineKind::Remove {
                    new = new.saturating_add(1);
                }
            }
        }
    }
    (old, new)
}

fn hunk_header(old_start: usize, old_len: usize, new_start: usize, new_len: usize) -> String {
    format!("@@ -{old_start},{old_len} +{new_start},{new_len} @@")
}

fn unified_lines(source: &dyn DiffSource, out: &mut Vec<OwnedLine>) {
    out.push(header(source, DiffMode::Unified));
    let digits = number_width(source);
    let mut old = 0usize;
    let mut new = 0usize;
    for index in 0..source.row_count() {
        match source.row(index) {
            Some(DiffRow::Hunk {
                old_start,
                new_start,
            }) => {
                old = old_start;
                new = new_start;
                let (old_len, new_len) = hunk_lengths(source, index);
                out.push(vec![OwnedRun::role(
                    hunk_header(old_start, old_len, new_start, new_len),
                    Role::Fg(FgStep::Muted),
                )]);
            }
            Some(DiffRow::Line { kind, text }) => {
                let (old_no, new_no, marker, role) = match kind {
                    DiffLineKind::Context => {
                        let row = (Some(old), Some(new), " ", Role::Fg(FgStep::Secondary));
                        old = old.saturating_add(1);
                        new = new.saturating_add(1);
                        row
                    }
                    DiffLineKind::Add => {
                        let row = (None, Some(new), "+", Role::Success);
                        new = new.saturating_add(1);
                        row
                    }
                    DiffLineKind::Remove => {
                        let row = (Some(old), None, "-", Role::Danger);
                        old = old.saturating_add(1);
                        row
                    }
                };
                out.push(vec![
                    OwnedRun::role(gutter(old_no, digits), Role::Fg(FgStep::Faint)),
                    OwnedRun::plain(" "),
                    OwnedRun::role(gutter(new_no, digits), Role::Fg(FgStep::Faint)),
                    OwnedRun::plain(" "),
                    OwnedRun::role(marker, role).bold(),
                    OwnedRun::role(format!(" {text}"), role),
                ]);
            }
            None => {}
        }
    }
    if source.row_count() == 0 {
        out.push(vec![OwnedRun::role(
            "(no textual changes)",
            Role::Fg(FgStep::Muted),
        )]);
    }
}

fn review_lines(source: &dyn DiffSource, width: u16, out: &mut Vec<OwnedLine>) {
    out.push(header(source, DiffMode::Review));
    let digits = number_width(source);
    let col = usize::from(width).saturating_sub(3) / 2;
    let text_width = col.saturating_sub(digits.saturating_add(1)).max(4);
    let mut old = 0usize;
    let mut new = 0usize;
    let mut index = 0usize;
    let mut hunk_index = 0usize;
    while index < source.row_count() {
        match source.row(index) {
            Some(DiffRow::Hunk {
                old_start,
                new_start,
            }) => {
                if hunk_index > 0 {
                    out.push(vec![OwnedRun::role(
                        "─".repeat(usize::from(width).min(200)),
                        Role::Fg(FgStep::Faint),
                    )]);
                }
                hunk_index = hunk_index.saturating_add(1);
                old = old_start;
                new = new_start;
                let (old_len, new_len) = hunk_lengths(source, index);
                out.push(vec![OwnedRun::role(
                    hunk_header(old_start, old_len, new_start, new_len),
                    Role::Fg(FgStep::Muted),
                )]);
                index = index.saturating_add(1);
            }
            Some(DiffRow::Line {
                kind: DiffLineKind::Context,
                text,
            }) => {
                let mut line = side(
                    old,
                    text,
                    Role::Fg(FgStep::Secondary),
                    text_width,
                    None,
                    digits,
                );
                line.push(OwnedRun::role(" │ ", Role::Fg(FgStep::Faint)));
                line.extend(side(
                    new,
                    text,
                    Role::Fg(FgStep::Secondary),
                    text_width,
                    None,
                    digits,
                ));
                out.push(line);
                old = old.saturating_add(1);
                new = new.saturating_add(1);
                index = index.saturating_add(1);
            }
            Some(DiffRow::Line {
                kind: DiffLineKind::Remove | DiffLineKind::Add,
                ..
            }) => {
                index =
                    append_changed_rows(source, index, text_width, digits, &mut old, &mut new, out);
            }
            None => index = index.saturating_add(1),
        }
    }
    if source.row_count() == 0 {
        out.push(vec![OwnedRun::role(
            "(no textual changes)",
            Role::Fg(FgStep::Muted),
        )]);
    }
}

fn append_changed_rows(
    source: &dyn DiffSource,
    start: usize,
    text_width: usize,
    digits: usize,
    old: &mut usize,
    new: &mut usize,
    out: &mut Vec<OwnedLine>,
) -> usize {
    let remove_start = start;
    let mut index = start;
    while matches!(
        source.row(index),
        Some(DiffRow::Line {
            kind: DiffLineKind::Remove,
            ..
        })
    ) {
        index = index.saturating_add(1);
    }
    let add_start = index;
    while matches!(
        source.row(index),
        Some(DiffRow::Line {
            kind: DiffLineKind::Add,
            ..
        })
    ) {
        index = index.saturating_add(1);
    }
    let remove_count = add_start.saturating_sub(remove_start);
    let add_count = index.saturating_sub(add_start);
    for pair in 0..remove_count.max(add_count) {
        let removed_text = source
            .row(remove_start.saturating_add(pair))
            .and_then(line_text);
        let added_text = source
            .row(add_start.saturating_add(pair))
            .and_then(line_text);
        let changed = removed_text
            .zip(added_text)
            .map(|(left, right)| changed_range(left, right));
        let mut line = match removed_text {
            Some(text) => side(*old, text, Role::Danger, text_width, changed, digits),
            None => blank_side(text_width, digits),
        };
        line.push(OwnedRun::role(" │ ", Role::Fg(FgStep::Faint)));
        line.extend(match added_text {
            Some(text) => side(*new, text, Role::Success, text_width, changed, digits),
            None => blank_side(text_width, digits),
        });
        if removed_text.is_some() {
            *old = old.saturating_add(1);
        }
        if added_text.is_some() {
            *new = new.saturating_add(1);
        }
        out.push(line);
    }
    index
}

fn line_text(row: DiffRow<'_>) -> Option<&str> {
    match row {
        DiffRow::Line { text, .. } => Some(text),
        DiffRow::Hunk { .. } => None,
    }
}

fn side(
    number: usize,
    text: &str,
    role: Role,
    text_width: usize,
    changed: Option<(usize, usize)>,
    digits: usize,
) -> OwnedLine {
    let mut line = vec![OwnedRun::role(
        format!("{number:>digits$} "),
        Role::Fg(FgStep::Faint),
    )];
    let clipped = truncate(text, text_width.min(usize::from(u16::MAX)) as u16);
    match changed {
        Some((prefix, suffix)) => {
            let end = clipped
                .len()
                .saturating_sub(suffix)
                .max(prefix)
                .min(clipped.len());
            let prefix = floor_boundary(&clipped, prefix.min(clipped.len()));
            let end = floor_boundary(&clipped, end).max(prefix);
            if prefix > 0 {
                line.push(OwnedRun::role(&clipped[..prefix], role));
            }
            if end > prefix {
                line.push(OwnedRun::role(&clipped[prefix..end], role).bold());
            }
            if end < clipped.len() {
                line.push(OwnedRun::role(&clipped[end..], role));
            }
        }
        None => line.push(OwnedRun::role(clipped, role)),
    }
    line
}

fn blank_side(text_width: usize, digits: usize) -> OwnedLine {
    vec![OwnedRun::plain(
        " ".repeat(digits.saturating_add(1).saturating_add(text_width)),
    )]
}

fn changed_range(left: &str, right: &str) -> (usize, usize) {
    let mut prefix = 0usize;
    for (l, r) in left.chars().zip(right.chars()) {
        if l != r {
            break;
        }
        prefix = prefix.saturating_add(l.len_utf8());
    }
    let left_tail = left.get(prefix..).unwrap_or("");
    let right_tail = right.get(prefix..).unwrap_or("");
    let mut suffix = 0usize;
    for (l, r) in left_tail.chars().rev().zip(right_tail.chars().rev()) {
        if l != r {
            break;
        }
        suffix = suffix.saturating_add(l.len_utf8());
    }
    (prefix, suffix.min(left.len().saturating_sub(prefix)))
}

fn floor_boundary(text: &str, mut offset: usize) -> usize {
    while offset > 0 && !text.is_char_boundary(offset) {
        offset = offset.saturating_sub(1);
    }
    offset
}

fn number_width(source: &dyn DiffSource) -> usize {
    let mut max = 1usize;
    for row in source_rows(source) {
        if let DiffRow::Hunk {
            old_start,
            new_start,
        } = row
        {
            max = max.max(old_start).max(new_start);
        }
    }
    decimal_digits(max).max(3)
}

fn decimal_digits(mut number: usize) -> usize {
    let mut digits = 1usize;
    while number >= 10 {
        digits = digits.saturating_add(1);
        number /= 10;
    }
    digits
}

fn gutter(number: Option<usize>, digits: usize) -> String {
    number.map_or_else(|| " ".repeat(digits), |number| format!("{number:>digits$}"))
}

#[cfg(test)]
mod tests {
    use ratatui_core::buffer::Buffer;

    use super::*;
    use crate::runtime::Runtime;
    use crate::runtime::stub::Stub;
    use crate::theme::Theme;
    use crate::{ReferenceState, ReferenceTarget};

    const ID: Id = Id::root("diff.tests");
    const ROWS: &[DiffRow<'static>] = &[
        DiffRow::Hunk {
            old_start: 10,
            new_start: 10,
        },
        DiffRow::Line {
            kind: DiffLineKind::Context,
            text: "fn retry() {",
        },
        DiffRow::Line {
            kind: DiffLineKind::Remove,
            text: "    let attempts = 3;",
        },
        DiffRow::Line {
            kind: DiffLineKind::Add,
            text: "    let attempts = 5;",
        },
        DiffRow::Line {
            kind: DiffLineKind::Add,
            text: "    let backoff = Backoff::exponential();",
        },
        DiffRow::Line {
            kind: DiffLineKind::Context,
            text: "}",
        },
        DiffRow::Hunk {
            old_start: 40,
            new_start: 41,
        },
        DiffRow::Line {
            kind: DiffLineKind::Remove,
            text: "// old",
        },
        DiffRow::Line {
            kind: DiffLineKind::Context,
            text: "done",
        },
    ];

    struct Source;

    impl DiffSource for Source {
        fn revision(&self) -> u64 {
            1
        }
        fn path(&self) -> &'static str {
            "src/settlement/retry.rs"
        }
        fn status_marker(&self) -> &'static str {
            "M"
        }
        fn status_label(&self) -> &'static str {
            "modified"
        }
        fn status_role(&self) -> Role {
            Role::Warning
        }
        fn row_count(&self) -> usize {
            ROWS.len()
        }
        fn row(&self, index: usize) -> Option<DiffRow<'_>> {
            ROWS.get(index).copied()
        }
    }

    fn text(line: &OwnedLine) -> String {
        line.iter().map(|run| run.text.as_str()).collect()
    }

    #[test]
    fn counts_and_headers() {
        assert_eq!(counts(&Source), (2, 2, 2));
        assert_eq!(hunk_lengths(&Source, 0), (3, 4));
        assert_eq!(hunk_header(10, 3, 10, 4), "@@ -10,3 +10,4 @@");
        assert_eq!(
            text(&header(&Source, DiffMode::Unified))
                .split("  +")
                .next(),
            Some("M src/settlement/retry.rs")
        );
    }

    #[test]
    fn unified_lists_every_line_with_markers() {
        let mut lines = Vec::new();
        unified_lines(&Source, &mut lines);
        let lines: Vec<String> = lines.iter().map(text).collect();
        assert!(lines[0].starts_with("M src/settlement/retry.rs"));
        assert_eq!(lines[1], "@@ -10,3 +10,4 @@");
        assert!(lines[3].contains("- ") && lines[3].contains("attempts = 3"));
        assert!(lines[4].contains("+ ") && lines[4].contains("attempts = 5"));
        assert!(lines[2].starts_with(" 10  10"));
    }

    #[test]
    fn review_pairs_columns_and_emphasises_the_change() {
        let mut lines = Vec::new();
        review_lines(&Source, 120, &mut lines);
        let texts: Vec<String> = lines.iter().map(text).collect();
        assert!(texts[3].contains('│'));
        assert!(texts[3].contains("attempts = 3") && texts[3].contains("attempts = 5"));
        let bold: Vec<&str> = lines[3]
            .iter()
            .filter(|run| run.modifier.contains(Modifier::BOLD))
            .map(|run| run.text.as_str())
            .collect();
        assert!(bold.iter().any(|run| run.contains('3')));
        assert!(bold.iter().any(|run| run.contains('5')));
        assert!(texts.iter().any(|line| line.starts_with("────")));
    }

    #[test]
    fn view_renders_and_scrolls_without_draw_mutation() {
        let view = DiffView::new(ID, Some(&Source));
        let mut state = DiffViewState::default();
        state.viewport.set_follow(false);
        let before = state.clone();
        let area = Rect::new(0, 0, 60, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            view.draw(ui, rect, &state);
        });
        assert_eq!(state, before);
        let row: String = buffer
            .content()
            .iter()
            .take(60)
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(row.contains("M src/settlement/retry.rs"), "{row:?}");
        assert_eq!(state.toggle_mode(), DiffMode::Review);
    }

    #[test]
    fn source_revision_and_review_width_key_the_runtime_cache() {
        let mut layout = DiffLayout::default();
        assert!(layout.ensure(Some(&Source), DiffMode::Unified, 80));
        let first = layout.scratch.clone();
        assert!(!layout.ensure(Some(&Source), DiffMode::Unified, 80));
        assert_eq!(layout.scratch, first);
        assert!(layout.ensure(Some(&Source), DiffMode::Review, 40));
        assert_ne!(layout.scratch, first);
        assert_eq!(layout.width, 40);
    }

    #[test]
    fn reference_mode_makes_the_composed_viewport_inert() {
        let view = DiffView::new(ID, Some(&Source));
        let state = DiffViewState::default();
        let area = Rect::new(0, 0, 40, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime.draw_scene(area, &mut buffer, |ui, rect| {
            ui.reference(
                Some(ReferenceTarget::new(ID, ReferenceState::FOCUSED)),
                |ui| {
                    view.draw(ui, rect, &state);
                },
            );
        });
        assert!(
            !runtime.ring().is_registered(ID),
            "reference mode must leave the nested viewport inert"
        );
    }
}
