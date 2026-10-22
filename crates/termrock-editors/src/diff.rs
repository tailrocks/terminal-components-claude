use core::fmt;

use ratatui_core::layout::Rect;
use ratatui_core::style::Modifier;

use super::SlotFn;
use crate::id::{Id, Part};
use crate::keymap::{Binding, BindingState, Bindings};
use crate::measure::{Constraints, Size};
use crate::response::Response;
use crate::text::truncate;
use crate::theme::{FgStep, Role, StylePatch};
use crate::ui::{Cx, FrameRead, Ui};
use termrock_viewport::viewport::{
    ProjectedText, TextViewport, ViewportAction, ViewportCmd, ViewportState,
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffLineKind {
    Context,

    Add,

    Remove,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DiffRow<'a> {
    Hunk { old_start: usize, new_start: usize },

    Line { kind: DiffLineKind, text: &'a str },
}

pub trait DiffSource {
    fn revision(&self) -> u64;

    fn path(&self) -> &str;

    fn status_marker(&self) -> &str;

    fn status_label(&self) -> &str;

    fn status_role(&self) -> Role {
        Role::Fg(FgStep::Secondary)
    }

    fn renamed_from(&self) -> Option<&str> {
        None
    }

    fn row_count(&self) -> usize;

    fn row(&self, index: usize) -> Option<DiffRow<'_>>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum DiffMode {
    #[default]
    Unified,

    Review,
}

impl DiffMode {
    pub const fn label(self) -> &'static str {
        match self {
            DiffMode::Unified => "unified",
            DiffMode::Review => "review",
        }
    }

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
    pub const fn mode(&self) -> DiffMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: DiffMode) {
        if self.mode != mode {
            self.mode = mode;
        }
    }

    pub fn toggle_mode(&mut self) -> DiffMode {
        self.set_mode(self.mode.toggled());
        self.mode
    }

    pub const fn viewport(&self) -> &ViewportState {
        &self.viewport
    }

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
    path: String,
    renamed_from: Option<String>,
    status_marker: String,
    status_label: String,
    status_role: Option<Role>,
    reset_pending: bool,
    scratch: Vec<OwnedLine>,
    projection: ProjectedText,
}

impl DiffLayout {
    fn ensure(&mut self, source: Option<&dyn DiffSource>, mode: DiffMode, width: u16) -> bool {
        let revision = source.map_or(0, DiffSource::revision);
        let has_source = source.is_some();
        let identity_matches = source.is_none_or(|source| {
            self.path == source.path()
                && self.renamed_from.as_deref() == source.renamed_from()
                && self.status_marker == source.status_marker()
                && self.status_label == source.status_label()
                && self.status_role == Some(source.status_role())
        });
        let source_changed = !self.valid
            || self.has_source != has_source
            || self.revision != revision
            || !identity_matches;
        if revision != u64::MAX
            && self.valid
            && self.revision == revision
            && self.mode == mode
            && self.width == width
            && self.has_source == has_source
            && identity_matches
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
        self.path = source.map_or_else(String::new, |source| source.path().to_owned());
        self.renamed_from = source.and_then(DiffSource::renamed_from).map(str::to_owned);
        self.status_marker =
            source.map_or_else(String::new, |source| source.status_marker().to_owned());
        self.status_label =
            source.map_or_else(String::new, |source| source.status_label().to_owned());
        self.status_role = source.map(DiffSource::status_role);
        self.reset_pending |= source_changed;
        true
    }
}

pub struct DiffView<'a> {
    id: Id,
    source: Option<&'a dyn DiffSource>,
    horizontal_pan: bool,
    patch: Option<&'a StylePatch>,
    parts: &'a [(Part, StylePatch)],
    slot: Option<(Part, SlotFn<'a>)>,
}

impl fmt::Debug for DiffView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DiffView")
            .field("id", &self.id)
            .field("source", &self.source.is_some())
            .field("horizontal_pan", &self.horizontal_pan)
            .field("parts", &self.parts.len())
            .field("slot", &self.slot.map(|(part, _)| part))
            .finish_non_exhaustive()
    }
}

impl<'a> DiffView<'a> {
    pub const PARTS: &'static [Part] = TextViewport::PARTS;

    pub const fn new(id: Id, source: Option<&'a dyn DiffSource>) -> Self {
        DiffView {
            id,
            source,
            horizontal_pan: false,
            patch: None,
            parts: &[],
            slot: None,
        }
    }

    pub const fn id(&self) -> Id {
        self.id
    }

    /// Enable the shared viewport's horizontal pan controls for long diff lines.
    /// The default is off, preserving the existing DiffView key and frame surface.
    #[must_use]
    pub const fn horizontal_pan(mut self, enabled: bool) -> Self {
        self.horizontal_pan = enabled;
        self
    }

    #[must_use]
    pub const fn patch(mut self, patch: &'a StylePatch) -> Self {
        self.patch = Some(patch);
        self
    }

    #[must_use]
    pub const fn patch_part(mut self, parts: &'a [(Part, StylePatch)]) -> Self {
        self.parts = parts;
        self
    }

    #[must_use]
    pub const fn slot(mut self, part: Part, slot: SlotFn<'a>) -> Self {
        self.slot = Some((part, slot));
        self
    }

    fn viewport(&self) -> TextViewport<'a> {
        let mut viewport = TextViewport::new(self.id)
            .horizontal_pan(self.horizontal_pan)
            .patch_part(self.parts);
        if let Some(patch) = self.patch {
            viewport = viewport.patch(patch);
        }
        if let Some((part, slot)) = self.slot {
            viewport = viewport.slot(part, slot);
        }
        viewport
    }

    pub fn update(&self, cx: &mut Cx<'_>, state: &mut DiffViewState) -> Response<ViewportAction> {
        let width = cx.area(self.id).map_or(80, |area| area.width);
        let mut layout = core::mem::take(cx.cache::<DiffLayout>(self.id));
        layout.ensure(self.source, state.mode, width);
        if layout.reset_pending {
            state.viewport.set_follow(false);
            state.viewport.invalidate();
            state.viewport.reset_scroll_offsets();
            let _ = state.viewport.clear_selection();
            layout.reset_pending = false;
        }
        let response =
            self.viewport()
                .update_projected(cx, &mut state.viewport, &layout.projection);
        *cx.cache::<DiffLayout>(self.id) = layout;
        response
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect, state: &DiffViewState) -> Rect {
        let mut layout = core::mem::take(ui.cache::<DiffLayout>(self.id));
        layout.ensure(self.source, state.mode, area.width);
        let mut viewport_state = state.viewport.clone();
        if layout.reset_pending {
            viewport_state.set_follow(false);
            viewport_state.invalidate();
            viewport_state.reset_scroll_offsets();
            let _ = viewport_state.clear_selection();
        }
        let used = self
            .viewport()
            .draw_projected(ui, area, &viewport_state, &layout.projection);
        *ui.cache::<DiffLayout>(self.id) = layout;
        used
    }

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
    use crate::event::{Axis, KeyCode, MouseKind};
    use crate::runtime::Runtime;
    use crate::runtime::stub::{Stub, key, mouse};
    use crate::theme::Theme;
    use crate::ui::PartRef;
    use crate::{ReferenceState, ReferenceTarget};
    use termrock_viewport::viewport::CellPos;

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

    #[derive(Clone, Copy)]
    struct NamedSource {
        revision: u64,
        path: &'static str,
        renamed_from: Option<&'static str>,
        marker: &'static str,
        label: &'static str,
        role: Role,
    }

    impl DiffSource for NamedSource {
        fn revision(&self) -> u64 {
            self.revision
        }

        fn path(&self) -> &str {
            self.path
        }

        fn status_marker(&self) -> &str {
            self.marker
        }

        fn status_label(&self) -> &str {
            self.label
        }

        fn status_role(&self) -> Role {
            self.role
        }

        fn renamed_from(&self) -> Option<&str> {
            self.renamed_from
        }

        fn row_count(&self) -> usize {
            ROWS.len()
        }

        fn row(&self, index: usize) -> Option<DiffRow<'_>> {
            ROWS.get(index).copied()
        }
    }

    const FIRST_NAMED_SOURCE: NamedSource = NamedSource {
        revision: 4,
        path: "src/first.rs",
        renamed_from: None,
        marker: "M",
        label: "modified",
        role: Role::Warning,
    };

    const SECOND_NAMED_SOURCE: NamedSource = NamedSource {
        revision: 4,
        path: "src/second.rs",
        renamed_from: None,
        marker: "A",
        label: "added",
        role: Role::Success,
    };

    struct DiffProbe {
        second_source: bool,
        state: DiffViewState,
    }

    impl DiffProbe {
        fn source(&self) -> &'static dyn DiffSource {
            if self.second_source {
                &SECOND_NAMED_SOURCE
            } else {
                &FIRST_NAMED_SOURCE
            }
        }
    }

    impl crate::runtime::App for DiffProbe {
        fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
            DiffView::new(ID, Some(self.source()))
                .horizontal_pan(true)
                .update(cx, &mut self.state)
                .erase()
        }

        fn draw(&self, ui: &mut Ui<'_>) {
            DiffView::new(ID, Some(self.source()))
                .horizontal_pan(true)
                .draw(ui, Rect::new(0, 0, 20, 6), &self.state);
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
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                view.draw(ui, rect, &state);
            })
            .commit_presented();
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
    fn cache_identity_includes_path_rename_and_header_metadata() {
        let first = NamedSource {
            revision: 7,
            path: "src/old.rs",
            renamed_from: None,
            marker: "M",
            label: "modified",
            role: Role::Warning,
        };
        let second = NamedSource {
            path: "src/new.rs",
            renamed_from: Some("src/old.rs"),
            ..first
        };
        let metadata = NamedSource {
            marker: "A",
            label: "added",
            role: Role::Success,
            ..second
        };
        let changed = NamedSource {
            revision: 8,
            ..metadata
        };
        let mut layout = DiffLayout::default();

        assert!(layout.ensure(Some(&first), DiffMode::Unified, 80));
        assert!(layout.reset_pending);
        assert!(text(layout.scratch.first().expect("header row")).contains("src/old.rs"));
        layout.reset_pending = false;
        assert!(!layout.ensure(Some(&first), DiffMode::Unified, 80));

        assert!(layout.ensure(Some(&second), DiffMode::Unified, 80));
        assert!(
            layout.reset_pending,
            "a new file identity requests a state reset"
        );
        let header_text = text(layout.scratch.first().expect("renamed header"));
        assert!(
            header_text.contains("src/old.rs → src/new.rs"),
            "{header_text}"
        );
        layout.reset_pending = false;

        assert!(layout.ensure(Some(&metadata), DiffMode::Unified, 80));
        assert!(text(layout.scratch.first().expect("status header")).starts_with("A "));
        layout.reset_pending = false;

        let label_only = NamedSource {
            label: "staged addition",
            ..metadata
        };
        assert!(layout.ensure(Some(&label_only), DiffMode::Unified, 80));
        layout.reset_pending = false;

        let role_only = NamedSource {
            role: Role::Danger,
            ..label_only
        };
        assert!(layout.ensure(Some(&role_only), DiffMode::Unified, 80));
        assert_eq!(layout.scratch[0][0].role, Some(Role::Danger));
        layout.reset_pending = false;

        assert!(layout.ensure(Some(&changed), DiffMode::Unified, 80));
        assert!(
            layout.reset_pending,
            "a changed source revision requests a state reset"
        );
        assert!(layout.ensure(None, DiffMode::Unified, 80));
        assert!(
            layout.reset_pending,
            "clearing the selected source requests a state reset"
        );
        assert!(layout.ensure(Some(&first), DiffMode::Unified, 80));
        assert!(
            layout.reset_pending,
            "switching back rebuilds the previous source"
        );
    }

    #[test]
    fn max_revision_disables_projection_reuse_without_rearming_source_reset() {
        let source = NamedSource {
            revision: u64::MAX,
            path: "src/max.rs",
            renamed_from: None,
            marker: "M",
            label: "modified",
            role: Role::Warning,
        };
        let mut layout = DiffLayout::default();
        assert!(layout.ensure(Some(&source), DiffMode::Unified, 80));
        assert!(layout.reset_pending);
        layout.reset_pending = false;
        assert!(layout.ensure(Some(&source), DiffMode::Unified, 80));
        assert!(!layout.reset_pending);
    }

    #[test]
    fn horizontal_pan_is_opt_in_on_diff_view() {
        assert!(!DiffView::new(ID, Some(&Source)).horizontal_pan);
        assert!(
            DiffView::new(ID, Some(&Source))
                .horizontal_pan(true)
                .horizontal_pan
        );
    }

    #[test]
    fn diff_horizontal_pan_reaches_the_shared_viewport_only_when_enabled() {
        let area = Rect::new(0, 0, 20, 6);
        let horizontal_track = PartRef::of(Part::custom("viewport.horizontal-track"));
        let draw_has_horizontal_track = |view: DiffView<'_>| {
            let mut runtime = Runtime::new(Stub::default(), Theme::junie());
            let mut buffer = Buffer::empty(area);
            runtime
                .draw_scene(area, &mut buffer, |ui, rect| {
                    view.draw(ui, rect, &DiffViewState::default());
                })
                .commit_presented();
            runtime.area_of_part(ID, horizontal_track).is_some()
        };

        assert!(!draw_has_horizontal_track(DiffView::new(ID, Some(&Source))));
        assert!(draw_has_horizontal_track(
            DiffView::new(ID, Some(&Source)).horizontal_pan(true)
        ));
    }

    #[test]
    fn source_change_resets_both_offsets_before_the_first_new_draw() {
        let mut state = DiffViewState::default();
        state.viewport.set_follow(false);
        let app = DiffProbe {
            second_source: false,
            state,
        };
        let area = Rect::new(0, 0, 20, 6);
        let mut runtime = Runtime::new(app, Theme::junie());
        let _ = runtime.initialize();
        let mut buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Right));
        assert_eq!(runtime.app().state.viewport.horizontal_offset(), 4);
        let _ =
            crate::runtime::stub::deliver(&mut runtime, mouse(MouseKind::Wheel(Axis::V, 1), 5, 2));
        assert!(runtime.app().state.viewport.scroll().offset() > 0);
        runtime
            .app_mut()
            .state
            .viewport
            .select(CellPos::new(1, 1), CellPos::new(1, 5));
        runtime.app_mut().second_source = true;

        buffer = Buffer::empty(area);
        runtime.draw_buffer(area, &mut buffer).commit_presented();
        let new_header: String = buffer
            .content()
            .iter()
            .take(usize::from(area.width))
            .map(ratatui_core::buffer::Cell::symbol)
            .collect();
        assert!(new_header.contains("A src/second.rs"), "{new_header:?}");
        assert_eq!(runtime.app().state.viewport.horizontal_offset(), 4);
        assert_eq!(runtime.app().state.viewport.scroll().offset(), 3);

        let _ = crate::runtime::stub::deliver(&mut runtime, key(KeyCode::Char('z')));
        assert_eq!(runtime.app().state.viewport.horizontal_offset(), 0);
        assert_eq!(runtime.app().state.viewport.scroll().offset(), 0);
        assert_eq!(runtime.app().state.viewport.selection(), None);
    }

    #[test]
    fn reference_mode_makes_the_composed_viewport_inert() {
        let view = DiffView::new(ID, Some(&Source));
        let state = DiffViewState::default();
        let area = Rect::new(0, 0, 40, 4);
        let mut runtime = Runtime::new(Stub::default(), Theme::junie());
        let mut buffer = Buffer::empty(area);
        runtime
            .draw_scene(area, &mut buffer, |ui, rect| {
                ui.reference(
                    Some(ReferenceTarget::new(ID, ReferenceState::FOCUSED)),
                    |ui| {
                        view.draw(ui, rect, &state);
                    },
                );
            })
            .commit_presented();
        assert!(
            !runtime.ring().is_registered(ID),
            "reference mode must leave the nested viewport inert"
        );
    }
}
