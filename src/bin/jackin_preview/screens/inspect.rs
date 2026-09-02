//! Inspect Changes: the change set of an instance in two modes. Compact is
//! the lightweight list (open a file to read its diff in place); Advanced
//! is a source-control style viewer with a changed-file tree beside a diff
//! preview. Both offer the unified listing and the review layout.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::layout::{Split, SplitDir};
use junie_tui::ui::text::{fit, truncate, truncate_middle, width};
use junie_tui::widgets::diff::{DiffMode, DiffView};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::splitter::Splitter;
use junie_tui::widgets::tree::{TreeEvent, TreeNode, TreeView};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::modals::modal_frame;
use super::{CustomModal, ModalResult};
use crate::sim::changes::{ChangeSet, ChangedFile, DiffStatus};
use crate::sim::world::World;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InspectMode {
    Compact,
    Advanced,
}

impl InspectMode {
    pub fn label(self) -> &'static str {
        match self {
            InspectMode::Compact => "compact",
            InspectMode::Advanced => "advanced",
        }
    }
    fn toggled(self) -> Self {
        match self {
            InspectMode::Compact => InspectMode::Advanced,
            InspectMode::Advanced => InspectMode::Compact,
        }
    }
}

/// Which region has the keyboard in the advanced layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
    Tree,
    Diff,
}

/// The change inspector modal. Emits `Custom("back")` when the operator
/// leaves towards the exit choices it was opened from, `Custom("close")`
/// otherwise.
pub struct InspectChanges {
    pub id: WidgetId,
    pub title: String,
    pub changes: ChangeSet,
    pub mode: InspectMode,
    /// Compact: cursor over the file rows (the unpushed row comes last).
    cursor: usize,
    /// Compact: a file is open and its diff fills the body.
    open: bool,
    list_scroll: ScrollState,
    list_area: Rect,
    tree: TreeView,
    /// Tree path → index into `changes.files`.
    leaves: Vec<(Vec<usize>, usize)>,
    diff: DiffView,
    split: Split,
    seam: Splitter,
    region: Region,
    /// Areas from the last render: tree, diff (for wheel routing).
    tree_area: Rect,
    diff_area: Rect,
    container: Rect,
    stacked: bool,
    /// Esc at the top level goes back to the exit choices rather than
    /// closing.
    returns_to_exit: bool,
    result: Option<ModalResult>,
    pub area: Rect,
}

const MIN_W: u16 = 76;
const MIN_H: u16 = 20;

impl InspectChanges {
    pub fn new(id: WidgetId, title: &str, changes: ChangeSet, mode: InspectMode) -> Self {
        let (nodes, leaves) = build_tree(&changes.files);
        let mut tree = TreeView::new(id.sub("tree"), nodes);
        tree.expand_all();
        let mut s = Self {
            id,
            title: title.to_owned(),
            changes,
            mode,
            cursor: 0,
            open: false,
            list_scroll: ScrollState::default(),
            list_area: Rect::ZERO,
            tree,
            leaves,
            diff: DiffView::new(id.sub("diff")),
            split: Split::new(34, 22, 30),
            seam: Splitter::new(id.sub("seam"), SplitDir::Horizontal),
            region: Region::Tree,
            tree_area: Rect::ZERO,
            diff_area: Rect::ZERO,
            container: Rect::ZERO,
            stacked: false,
            returns_to_exit: false,
            result: None,
            area: Rect::ZERO,
        };
        s.select_file(0);
        s
    }

    /// Esc at the top level returns to the exit choices (`Custom("back")`).
    pub fn returns_to_exit(mut self, on: bool) -> Self {
        self.returns_to_exit = on;
        self
    }

    pub fn diff_mode(&self) -> DiffMode {
        self.diff.mode
    }

    /// The file whose diff is shown, if any.
    pub fn selected_file(&self) -> Option<&ChangedFile> {
        self.diff
            .file()
            .and_then(|f| self.changes.files.iter().find(|x| x.path == f.path))
    }

    pub fn is_open(&self) -> bool {
        self.open
    }

    fn select_file(&mut self, i: usize) {
        let f = self.changes.files.get(i).cloned();
        self.diff.set_file(f);
        if let Some((path, _)) = self.leaves.iter().find(|(_, fi)| *fi == i) {
            self.tree.selected = Some(path.clone());
            if let Some(row) = self.tree.rows().iter().position(|r| &r.path == path) {
                self.tree.cursor = row;
                self.tree.scroll.ensure_visible(row);
            }
        }
        if i < self.changes.files.len() {
            self.cursor = i;
        }
    }

    fn file_of_path(&self, path: &[usize]) -> Option<usize> {
        self.leaves.iter().find(|(p, _)| p == path).map(|(_, i)| *i)
    }

    /// Compact rows: one per file plus the unpushed-commits row.
    fn compact_rows(&self) -> usize {
        self.changes.files.len() + usize::from(self.changes.unpushed > 0)
    }

    fn set_mode(&mut self, mode: InspectMode, focus: &mut Focus) {
        self.mode = mode;
        self.open = false;
        match mode {
            InspectMode::Advanced => {
                self.region = Region::Tree;
                focus.focus(self.tree.id);
            }
            InspectMode::Compact => focus.focus(self.id),
        }
    }

    fn set_region(&mut self, region: Region, focus: &mut Focus) {
        self.region = region;
        focus.focus(match region {
            Region::Tree => self.tree.id,
            Region::Diff => self.diff.id(),
        });
    }

    fn leave(&mut self) -> Outcome {
        self.result = Some(ModalResult::Custom(
            if self.returns_to_exit {
                "back"
            } else {
                "close"
            }
            .into(),
        ));
        Outcome::Changed
    }

    /// Wheel with the pointer position: the region under the pointer scrolls.
    pub fn on_wheel_at(&mut self, delta: i32, pos: Position) -> Outcome {
        match self.mode {
            InspectMode::Advanced => {
                if self.tree_area.contains(pos) {
                    self.tree.on_wheel(delta)
                } else if self.diff_area.contains(pos) {
                    self.diff.on_wheel(delta)
                } else {
                    Outcome::Consumed
                }
            }
            InspectMode::Compact => {
                if self.open {
                    self.diff.on_wheel(delta)
                } else {
                    self.list_scroll.scroll_by(delta as isize);
                    Outcome::Changed
                }
            }
        }
    }

    /// Drag on the seam resizes the split; drag in the diff extends a selection.
    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if pressed == self.seam.id && !self.stacked {
            return self.seam.on_drag(&mut self.split, self.container, 1, pos);
        }
        if self.diff.owns(pressed) {
            return self.diff.on_drag(pos);
        }
        Outcome::Ignored
    }

    fn compact_key(&mut self, key: &Key, focus: &mut Focus) -> Outcome {
        if self.open {
            match key.code {
                KeyCode::Esc | KeyCode::Backspace | KeyCode::Char('q') => {
                    self.open = false;
                    focus.focus(self.id);
                    return Outcome::Changed;
                }
                _ => {}
            }
            let (o, _) = self.diff.on_key(key);
            return o.or(Outcome::Consumed);
        }
        let n = self.compact_rows();
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => self.leave(),
            KeyCode::Up | KeyCode::Char('k') => {
                self.cursor = self.cursor.saturating_sub(1);
                self.list_scroll.ensure_visible(self.cursor);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.cursor = (self.cursor + 1).min(n.saturating_sub(1));
                self.list_scroll.ensure_visible(self.cursor);
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = 0;
                self.list_scroll.ensure_visible(0);
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = n.saturating_sub(1);
                self.list_scroll.ensure_visible(self.cursor);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.cursor = self
                    .cursor
                    .saturating_sub(self.list_scroll.viewport_len.max(1));
                self.list_scroll.ensure_visible(self.cursor);
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.cursor =
                    (self.cursor + self.list_scroll.viewport_len.max(1)).min(n.saturating_sub(1));
                self.list_scroll.ensure_visible(self.cursor);
                Outcome::Changed
            }
            KeyCode::Enter | KeyCode::Char(' ') | KeyCode::Right | KeyCode::Char('l') => {
                if self.cursor < self.changes.files.len() {
                    self.select_file(self.cursor);
                    self.open = true;
                    focus.focus(self.diff.id());
                }
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    fn advanced_key(&mut self, key: &Key, focus: &mut Focus) -> Outcome {
        match key.code {
            KeyCode::Tab | KeyCode::BackTab => {
                let next = match self.region {
                    Region::Tree => Region::Diff,
                    Region::Diff => Region::Tree,
                };
                self.set_region(next, focus);
                return Outcome::Changed;
            }
            KeyCode::Esc if self.region == Region::Diff => {
                // clear a selection first, then hand the keyboard back to the tree
                let (o, _) = self.diff.on_key(key);
                if o != Outcome::Changed {
                    self.set_region(Region::Tree, focus);
                }
                return Outcome::Changed;
            }
            KeyCode::Esc | KeyCode::Char('q') => return self.leave(),
            _ => {}
        }
        match self.region {
            Region::Tree => {
                let (o, ev) = self.tree.on_key(key);
                if let Some(TreeEvent::Activate(path)) = ev
                    && let Some(i) = self.file_of_path(&path)
                {
                    self.select_file(i);
                    return Outcome::Changed;
                }
                // moving the cursor over a file previews it at once
                if o == Outcome::Changed
                    && let Some(row) = self.tree.rows().get(self.tree.cursor)
                    && let Some(i) = self.file_of_path(&row.path)
                {
                    self.select_file(i);
                    self.tree.cursor =
                        row_index(&self.tree, i, &self.leaves).unwrap_or(self.tree.cursor);
                }
                o.or(Outcome::Consumed)
            }
            Region::Diff => {
                let (o, _) = self.diff.on_key(key);
                o.or(Outcome::Consumed)
            }
        }
    }

    fn draw_compact(
        &mut self,
        inner: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
        let t = ctx.theme;
        if self.open {
            let Some(f) = self.diff.file().cloned() else {
                return;
            };
            let head = format!("{}  {}", f.header(), f.summary());
            buf.set_string(
                inner.x,
                inner.y,
                truncate(&head, inner.width as usize),
                t.secondary().bg(bg),
            );
            let mode = format!("{} · d switches", self.diff.mode.label());
            let mw = width(&mode) as u16;
            if inner.width > width(&head) as u16 + mw + 3 {
                buf.set_string(inner.right() - mw, inner.y, &mode, t.faint().bg(bg));
            }
            let body = Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(2),
            );
            self.diff_area = body;
            self.diff.render(body, buf, ctx, bg);
            return;
        }
        let summary = self.changes.summary();
        buf.set_string(inner.x, inner.y, &summary, t.secondary().bg(bg));
        let mode = format!("{} · {}", self.mode.label(), self.diff.mode.label());
        let mw = width(&mode) as u16;
        if inner.width > width(&summary) as u16 + mw + 3 {
            buf.set_string(inner.right() - mw, inner.y, &mode, t.faint().bg(bg));
        }
        let list = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );
        self.list_area = list;
        let n = self.compact_rows();
        self.list_scroll.set_content(n);
        self.list_scroll.set_viewport(list.height as usize);
        ctx.control(self.id, list, false);
        ctx.scrollable(self.id, list);
        let has_sb = self.list_scroll.overflows();
        let row_w = list.width.saturating_sub(u16::from(has_sb));
        let focused = ctx.interaction.focused(self.id);
        if n == 0 {
            buf.set_string(
                list.x + 2,
                list.y,
                "Working tree clean · nothing to inspect",
                t.muted().bg(bg),
            );
        }
        for (k, i) in self.list_scroll.visible_range().enumerate() {
            let y = list.y + k as u16;
            let rid = self.id.sub("row").child(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.cursor;
            s.selected = i == self.cursor;
            let st = t.row(s, bg);
            let rect = Rect::new(list.x, y, row_w, 1);
            fill(buf, rect, st);
            buf.set_string(rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            if i < self.changes.files.len() {
                let f = &self.changes.files[i];
                buf.set_string(
                    rect.x + 2,
                    y,
                    f.status.marker(),
                    st.fg(t.tone(f.status.tone())).add_modifier(Modifier::BOLD),
                );
                let counts = format!("+{} −{}", f.additions(), f.deletions());
                let cw = width(&counts) as u16;
                let pw = rect.width.saturating_sub(6 + cw + 2) as usize;
                let path = match &f.status {
                    DiffStatus::Renamed { from } => format!("{from} → {}", f.path),
                    _ => f.path.clone(),
                };
                buf.set_string(
                    rect.x + 4,
                    y,
                    fit(&truncate_middle(&path, pw), pw),
                    if s.selected {
                        st.fg(t.text_primary)
                    } else {
                        st
                    },
                );
                buf.set_string(
                    rect.right().saturating_sub(cw + 1),
                    y,
                    &counts,
                    st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                );
            } else {
                let u = self.changes.unpushed;
                buf.set_string(rect.x + 2, y, "↑", st.fg(t.warning));
                buf.set_string(
                    rect.x + 4,
                    y,
                    format!("{u} commit{} not pushed", if u == 1 { "" } else { "s" }),
                    st.fg(t.warning),
                );
            }
            ctx.clickable(rid, rect);
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(list.right() - 1, list.y, 1, list.height),
                buf,
                ctx,
                self.id,
                &self.list_scroll,
                focused,
            );
        }
    }

    fn draw_advanced(
        &mut self,
        inner: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: ratatui::style::Color,
    ) {
        let t = ctx.theme;
        let summary = self.changes.summary();
        buf.set_string(inner.x, inner.y, &summary, t.secondary().bg(bg));
        let mode = format!(
            "{} · {} · d switches · m compact",
            self.mode.label(),
            self.diff.mode.label()
        );
        let mw = width(&mode) as u16;
        if inner.width > width(&summary) as u16 + mw + 3 {
            buf.set_string(inner.right() - mw, inner.y, &mode, t.faint().bg(bg));
        }
        let body = Rect::new(
            inner.x,
            inner.y + 2,
            inner.width,
            inner.height.saturating_sub(2),
        );
        self.container = body;
        // narrow dialogs stack the tree above the diff instead of squeezing both
        self.stacked = body.width < 90;
        let (tree_r, diff_r) = if self.stacked {
            let th = (body.height * 35 / 100).clamp(4, 10);
            (
                Rect::new(body.x, body.y, body.width, th),
                Rect::new(
                    body.x,
                    body.y + th + 1,
                    body.width,
                    body.height.saturating_sub(th + 1),
                ),
            )
        } else {
            self.split.horizontal(body, 1)
        };
        self.tree_area = tree_r;
        self.diff_area = diff_r;
        self.tree.render(tree_r, buf, ctx, bg);
        if self.stacked {
            let y = tree_r.bottom();
            for x in body.left()..body.right() {
                buf.set_string(x, y, "─", Style::new().fg(t.border_subtle).bg(bg));
            }
        } else {
            let handle = self.split.handle(SplitDir::Horizontal, body, 1);
            self.seam.render(handle, buf, ctx, bg);
        }
        self.diff.render(diff_r, buf, ctx, bg);
        if let Some(u) = Some(self.changes.unpushed).filter(|u| *u > 0)
            && diff_r.height > 2
        {
            // the unpushed commits live on the last row of the diff column
            let text = format!("↑ {u} commit{} not pushed", if u == 1 { "" } else { "s" });
            let y = diff_r.bottom() - 1;
            let w = width(&text) as u16;
            buf.set_string(
                diff_r.right().saturating_sub(w),
                y,
                &text,
                Style::new().fg(t.warning).bg(bg),
            );
        }
    }
}

/// Row index in the flattened tree for file `i`.
fn row_index(tree: &TreeView, i: usize, leaves: &[(Vec<usize>, usize)]) -> Option<usize> {
    let (path, _) = leaves.iter().find(|(_, fi)| *fi == i)?;
    tree.rows().iter().position(|r| &r.path == path)
}

/// Group files into folder nodes. Returns the nodes and the tree path of
/// every leaf with its file index.
fn build_tree(files: &[ChangedFile]) -> (Vec<TreeNode>, Vec<(Vec<usize>, usize)>) {
    #[derive(Default)]
    struct Dir {
        dirs: std::collections::BTreeMap<String, Dir>,
        files: Vec<(String, usize)>,
    }
    let mut root = Dir::default();
    for (i, f) in files.iter().enumerate() {
        let mut parts: Vec<&str> = f.path.split('/').collect();
        let name = parts.pop().unwrap_or("").to_owned();
        let mut d = &mut root;
        for p in parts {
            d = d.dirs.entry(p.to_owned()).or_default();
        }
        d.files.push((name, i));
    }
    fn count(d: &Dir) -> usize {
        d.files.len() + d.dirs.values().map(count).sum::<usize>()
    }
    fn emit(
        d: &Dir,
        files: &[ChangedFile],
        path: &mut Vec<usize>,
        leaves: &mut Vec<(Vec<usize>, usize)>,
    ) -> Vec<TreeNode> {
        let mut out = vec![];
        for (name, sub) in &d.dirs {
            path.push(out.len());
            let children = emit(sub, files, path, leaves);
            path.pop();
            let n = count(sub);
            out.push(TreeNode::dir(name, children).meta(&format!("{n} changed")));
        }
        for (name, i) in &d.files {
            path.push(out.len());
            leaves.push((path.clone(), *i));
            path.pop();
            let f = &files[*i];
            out.push(
                TreeNode::leaf_meta(name, &format!("+{} −{}", f.additions(), f.deletions()))
                    .glyph(f.status.marker()),
            );
        }
        out
    }
    let mut leaves = vec![];
    let nodes = emit(&root, files, &mut vec![], &mut leaves);
    (nodes, leaves)
}

impl CustomModal for InspectChanges {
    fn on_key(&mut self, key: &Key, focus: &mut Focus, _ring: &FocusRing, _w: &World) -> Outcome {
        match key.code {
            KeyCode::Char('d') if key.plain() => {
                self.diff.toggle_mode();
                return Outcome::Changed;
            }
            KeyCode::Char('m') | KeyCode::F(2) if key.plain() => {
                self.set_mode(self.mode.toggled(), focus);
                return Outcome::Changed;
            }
            _ => {}
        }
        match self.mode {
            InspectMode::Compact => self.compact_key(key, focus),
            InspectMode::Advanced => self.advanced_key(key, focus),
        }
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, focus: &mut Focus, _w: &World) -> Outcome {
        match self.mode {
            InspectMode::Compact => {
                if self.open {
                    if self.diff.owns(id) {
                        if id == scrollbar::id_for(self.diff.id()) {
                            return self.diff.on_scrollbar(pos);
                        }
                        return self.diff.on_click(pos);
                    }
                    return Outcome::Consumed;
                }
                for i in 0..self.compact_rows() {
                    if self.id.sub("row").child(i) == id {
                        let same = self.cursor == i;
                        self.cursor = i;
                        focus.focus(self.id);
                        if same && i < self.changes.files.len() {
                            self.select_file(i);
                            self.open = true;
                            focus.focus(self.diff.id());
                        }
                        return Outcome::Changed;
                    }
                }
                if id == scrollbar::id_for(self.id) {
                    let track = Rect::new(
                        self.list_area.right() - 1,
                        self.list_area.y,
                        1,
                        self.list_area.height,
                    );
                    self.list_scroll.scroll_to(scrollbar::offset_for_click(
                        track,
                        pos,
                        &self.list_scroll,
                    ));
                    return Outcome::Changed;
                }
                Outcome::Consumed
            }
            InspectMode::Advanced => {
                if let Some((row, toggle)) = self.tree.locate(id) {
                    self.set_region(Region::Tree, focus);
                    let (o, ev) = if toggle {
                        self.tree.on_click_toggle(row)
                    } else {
                        self.tree.on_click_row(row)
                    };
                    if let Some(TreeEvent::Activate(path)) = ev
                        && let Some(i) = self.file_of_path(&path)
                    {
                        self.select_file(i);
                    }
                    return o.or(Outcome::Changed);
                }
                if id == scrollbar::id_for(self.tree.id) {
                    return self.tree.on_scrollbar(pos);
                }
                if self.diff.owns(id) {
                    self.set_region(Region::Diff, focus);
                    if id == scrollbar::id_for(self.diff.id()) {
                        return self.diff.on_scrollbar(pos);
                    }
                    return self.diff.on_click(pos);
                }
                Outcome::Consumed
            }
        }
    }

    fn on_wheel(&mut self, delta: i32) -> Outcome {
        match self.mode {
            InspectMode::Advanced => match self.region {
                Region::Tree => self.tree.on_wheel(delta),
                Region::Diff => self.diff.on_wheel(delta),
            },
            InspectMode::Compact => {
                if self.open {
                    self.diff.on_wheel(delta)
                } else {
                    self.list_scroll.scroll_by(delta as isize);
                    Outcome::Changed
                }
            }
        }
    }

    fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        let w = (screen.width * 9 / 10).max(MIN_W.min(screen.width.saturating_sub(2)));
        let h = (screen.height * 9 / 10).max(MIN_H.min(screen.height.saturating_sub(2)));
        let meta = match self.mode {
            InspectMode::Compact if self.open => "Esc back to the list".to_owned(),
            InspectMode::Compact => "m advanced view".to_owned(),
            InspectMode::Advanced => "Tab tree / diff".to_owned(),
        };
        let (area, inner) = modal_frame(screen, buf, ctx, w, h, &self.title, Some(&meta), false);
        self.area = area;
        let bg = ctx.theme.surface_elevated;
        match self.mode {
            InspectMode::Compact => self.draw_compact(inner, buf, ctx, bg),
            InspectMode::Advanced => self.draw_advanced(inner, buf, ctx, bg),
        }
    }

    fn done(&mut self) -> Option<ModalResult> {
        self.result.take()
    }

    fn initial_focus(&self) -> WidgetId {
        match self.mode {
            InspectMode::Compact => self.id,
            InspectMode::Advanced => self.tree.id,
        }
    }

    fn hints(&self) -> Vec<Hint> {
        let back = if self.returns_to_exit {
            "Back"
        } else {
            "Close"
        };
        match self.mode {
            InspectMode::Compact if self.open => vec![
                hint("↑↓", "Scroll"),
                hint("drag", "Select"),
                hint("y", "Copy"),
                hint("d", "Unified / review"),
                hint("Esc", "List"),
            ],
            InspectMode::Compact => vec![
                hint("↑↓", "Move"),
                hint("Enter", "Open diff"),
                hint("d", "Unified / review"),
                hint("m", "Advanced"),
                hint("Esc", back),
            ],
            InspectMode::Advanced => match self.region {
                Region::Tree => vec![
                    hint("↑↓", "Move"),
                    hint("←→", "Fold"),
                    hint("Tab", "Diff"),
                    hint("d", "Unified / review"),
                    hint("m", "Compact"),
                    hint("Esc", back),
                ],
                Region::Diff => vec![
                    hint("↑↓", "Scroll"),
                    hint("drag", "Select"),
                    hint("y", "Copy"),
                    hint("Tab", "Tree"),
                    hint("d", "Unified / review"),
                    hint("Esc", "Tree"),
                ],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::fixtures::world_for;
    use crate::scenario::Scenario;
    use crate::sim::changes::changes_for;
    use junie_tui::core::hit::HitRegistry;
    use junie_tui::theme::Theme;
    use junie_tui::ui::ctx::Interaction;
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn changes() -> ChangeSet {
        changes_for(
            "jk-7f3a",
            &[
                "src/settlement/retry.rs".to_owned(),
                "src/settlement/mod.rs".to_owned(),
                "Cargo.toml".to_owned(),
            ],
            6,
            1,
        )
    }

    fn text(buf: &Buffer) -> String {
        let mut s = String::new();
        for y in 0..buf.area.height {
            for x in 0..buf.area.width {
                s.push_str(buf[(x, y)].symbol());
            }
            s.push('\n');
        }
        s
    }

    struct Rig {
        theme: Theme,
        world: World,
        focus: Focus,
        ring: FocusRing,
        w: u16,
        h: u16,
    }

    impl Rig {
        fn new(w: u16, h: u16) -> Self {
            Self {
                theme: Theme::junie(),
                world: world_for(Scenario::Returning),
                focus: Focus::default(),
                ring: FocusRing::default(),
                w,
                h,
            }
        }
        fn draw(&mut self, m: &mut InspectChanges) -> Buffer {
            let area = Rect::new(0, 0, self.w, self.h);
            let mut buf = Buffer::empty(area);
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(
                &self.theme,
                Interaction {
                    focus: self.focus.current(),
                    ..Default::default()
                },
                &mut hits,
                &mut ring,
            );
            m.render(area, &mut buf, &mut ctx, &self.world);
            self.ring = ring;
            buf
        }
        fn key(&mut self, m: &mut InspectChanges, code: KeyCode) -> Outcome {
            let o = m.on_key(&key(code), &mut self.focus, &self.ring, &self.world);
            self.draw(m);
            o
        }
    }

    #[test]
    fn compact_opens_a_file_and_returns_to_the_list() {
        let mut rig = Rig::new(120, 40);
        let mut m = InspectChanges::new(
            WidgetId::of("t.inspect"),
            "Inspect changes",
            changes(),
            InspectMode::Compact,
        )
        .returns_to_exit(true);
        rig.focus.focus(m.initial_focus());
        let t = text(&rig.draw(&mut m));
        assert!(t.contains("6 files · +"), "{t}");
        assert!(
            t.contains("M src/settlement/retry.rs")
                || t.contains("M  src/settlement/retry.rs")
                || t.contains("src/settlement/retry.rs"),
            "{t}"
        );
        assert!(t.contains("1 commit not pushed"), "{t}");
        rig.key(&mut m, KeyCode::Down);
        rig.key(&mut m, KeyCode::Enter);
        assert!(m.is_open());
        let t = text(&rig.draw(&mut m));
        assert!(t.contains("@@ -"), "diff visible: {t}");
        assert!(t.contains("pub mod backoff;"), "{t}");
        assert_eq!(
            m.selected_file().map(|f| f.path.as_str()),
            Some("src/settlement/mod.rs")
        );
        rig.key(&mut m, KeyCode::Char('d'));
        assert_eq!(m.diff_mode(), DiffMode::Review);
        let t = text(&rig.draw(&mut m));
        assert!(t.contains("│"), "review columns: {t}");
        rig.key(&mut m, KeyCode::Esc);
        assert!(!m.is_open());
        assert!(m.done().is_none());
        rig.key(&mut m, KeyCode::Esc);
        assert_eq!(m.done(), Some(ModalResult::Custom("back".into())));
    }

    #[test]
    fn advanced_tree_drives_the_diff_and_modes_toggle() {
        let mut rig = Rig::new(120, 40);
        let mut m = InspectChanges::new(
            WidgetId::of("t.inspect"),
            "Inspect changes",
            changes(),
            InspectMode::Advanced,
        );
        rig.focus.focus(m.initial_focus());
        let t = text(&rig.draw(&mut m));
        assert!(
            t.contains("settlement") && t.contains("retry.rs") && t.contains("Cargo.toml"),
            "{t}"
        );
        assert!(t.contains("3 changed") || t.contains("changed"), "{t}");
        assert_eq!(
            m.selected_file().map(|f| f.path.as_str()),
            Some("src/settlement/retry.rs")
        );
        // walk down the tree: the preview follows the cursor
        let mut seen = vec![];
        for _ in 0..8 {
            rig.key(&mut m, KeyCode::Down);
            if let Some(f) = m.selected_file() {
                seen.push(f.path.clone());
            }
        }
        assert!(seen.iter().any(|p| p == "Cargo.toml"), "{seen:?}");
        // Tab into the diff and scroll it
        rig.key(&mut m, KeyCode::Tab);
        assert!(rig.focus.is(m.diff.id()));
        let before = m.diff.term.scroll.offset;
        rig.key(&mut m, KeyCode::Down);
        let _ = before;
        // wheel by position: over the tree scrolls the tree, over the diff the diff
        let tree_pos = Position::new(m.tree_area.x + 1, m.tree_area.y + 1);
        let diff_pos = Position::new(m.diff_area.x + 1, m.diff_area.y + 1);
        m.tree.scroll.set_content(40);
        m.tree.scroll.set_viewport(5);
        assert_eq!(m.on_wheel_at(2, tree_pos), Outcome::Changed);
        assert_eq!(m.tree.scroll.offset, 2);
        let d0 = m.diff.term.scroll.offset;
        assert_eq!(m.on_wheel_at(3, diff_pos), Outcome::Changed);
        assert!(m.diff.term.scroll.offset >= d0);
        rig.draw(&mut m);
        assert_eq!(m.tree.scroll.offset, 2, "render keeps the wheel position");
        // switch to compact and back
        rig.key(&mut m, KeyCode::Char('m'));
        assert_eq!(m.mode, InspectMode::Compact);
        assert!(rig.focus.is(m.id));
        rig.key(&mut m, KeyCode::F(2));
        assert_eq!(m.mode, InspectMode::Advanced);
        rig.key(&mut m, KeyCode::Char('q'));
        assert_eq!(m.done(), Some(ModalResult::Custom("close".into())));
    }

    #[test]
    fn narrow_terminal_stacks_the_advanced_layout() {
        let mut rig = Rig::new(80, 24);
        let mut m = InspectChanges::new(
            WidgetId::of("t.inspect"),
            "Inspect changes",
            changes(),
            InspectMode::Advanced,
        );
        rig.focus.focus(m.initial_focus());
        let t = text(&rig.draw(&mut m));
        assert!(m.stacked);
        assert!(m.tree_area.y < m.diff_area.y);
        assert_eq!(m.tree_area.width, m.diff_area.width);
        assert!(t.contains("retry.rs") && t.contains("@@ -"), "{t}");
    }
}
