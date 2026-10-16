use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

/// A node in a tree: label, optional trailing metadata, children.
#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
    pub meta: Option<String>,
    /// One-cell muted glyph before the label (object kind).
    pub glyph: Option<&'static str>,
    /// Children are fetched on first expand; until then the node shows `▸`
    /// and expanding emits [`TreeEvent::Expand`].
    pub lazy: bool,
    /// A load is in flight (spinner instead of the fold glyph).
    pub busy: bool,
    /// Muted, not selectable, e.g. an empty-section note.
    pub note: bool,
}

impl TreeNode {
    pub fn leaf(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            children: vec![],
            meta: None,
            glyph: None,
            lazy: false,
            busy: false,
            note: false,
        }
    }

    pub fn leaf_meta(label: &str, meta: &str) -> Self {
        Self {
            meta: Some(meta.to_owned()),
            ..Self::leaf(label)
        }
    }

    pub fn dir(label: &str, children: Vec<TreeNode>) -> Self {
        Self {
            children,
            ..Self::leaf(label)
        }
    }

    /// A folder whose children arrive later via [`TreeView::set_children`].
    pub fn lazy(label: &str) -> Self {
        Self {
            lazy: true,
            ..Self::leaf(label)
        }
    }

    pub fn glyph(mut self, g: &'static str) -> Self {
        self.glyph = Some(g);
        self
    }

    pub fn note(label: &str) -> Self {
        Self {
            note: true,
            ..Self::leaf(label)
        }
    }

    pub fn meta(mut self, m: &str) -> Self {
        self.meta = Some(m.to_owned());
        self
    }
}

/// Path of child indices from the root.
pub type Path = Vec<usize>;

#[derive(Debug, Clone)]
pub struct FlatRow {
    pub path: Path,
    pub depth: usize,
    pub label: String,
    pub meta: Option<String>,
    pub has_children: bool,
    pub expanded: bool,
    pub glyph: Option<&'static str>,
    pub busy: bool,
    pub note: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeEvent {
    /// A lazy node was expanded: the owner should fetch and `set_children`.
    Expand(Path),
    /// Enter on a leaf.
    Activate(Path),
}

/// What Enter/Space (and a row click, see [`TreeView::row_click_toggles`])
/// do on a branch row.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BranchAction {
    /// Expand or collapse it (the long-standing behaviour).
    #[default]
    Toggle,
    /// Activate it like a leaf ([`TreeEvent::Activate`]); ←/→ and the
    /// disclosure still expand and collapse.
    Activate,
    /// Toggle it and activate it.
    Both,
}

#[derive(Debug, Clone)]
pub struct TreeView {
    /// The scrollbar track as last drawn: presses and drags map through it.
    track: Rect,
    pub id: WidgetId,
    pub nodes: Vec<TreeNode>,
    pub expanded: std::collections::HashSet<Path>,
    pub cursor: usize,
    pub selected: Option<Path>,
    pub scroll: ScrollState,
    pub area: Rect,
    rows: Vec<FlatRow>,
    /// Case-insensitive substring filter; ancestors of matches stay visible.
    pub filter: Option<String>,
    /// Cells kept free at the right of every row (left of the scrollbar)
    /// for columns the owner draws there (a tree table's metrics). Labels
    /// and metadata end before them and are cut with `…`, so the trailing
    /// columns never paint over a label. Default 0: the label row uses the
    /// full width.
    pub reserve_right: u16,
    /// Draw each row's trailing metadata. Default true; an owner that keeps
    /// a key in `meta` (and draws its own trailing columns) turns it off.
    pub show_meta: bool,
    /// The trailing column strip as last drawn (`x` and `width`; `y` and
    /// `height` span the rows), or `Rect::ZERO` when nothing is reserved or
    /// the strip does not fit. Owners align their headers and cells to it.
    trailing: Rect,
    /// What Enter/Space do on a branch row. Default
    /// [`BranchAction::Toggle`].
    pub branch_activate: BranchAction,
    /// A click on a branch row (not its disclosure) runs
    /// [`Self::branch_activate`]. Off: only the disclosure toggles, as in
    /// macOS outline views, and the click just moves the cursor. Default
    /// `true` (today's behaviour).
    pub row_click_toggles: bool,
    /// Columns of indentation per depth level. Default 2.
    pub indent: u16,
}

impl TreeView {
    pub fn new(id: WidgetId, nodes: Vec<TreeNode>) -> Self {
        let mut tv = Self {
            id,
            nodes,
            expanded: Default::default(),
            cursor: 0,
            selected: None,
            scroll: ScrollState::default(),
            area: Rect::ZERO,
            track: Rect::ZERO,
            rows: vec![],
            filter: None,
            reserve_right: 0,
            show_meta: true,
            trailing: Rect::ZERO,
            branch_activate: BranchAction::Toggle,
            row_click_toggles: true,
            indent: 2,
        };
        // expand the first level by default
        for i in 0..tv.nodes.len() {
            tv.expanded.insert(vec![i]);
        }
        tv.flatten();
        tv
    }

    pub fn branch_activate(mut self, action: BranchAction) -> Self {
        self.branch_activate = action;
        self
    }

    pub fn row_click_toggles(mut self, on: bool) -> Self {
        self.row_click_toggles = on;
        self
    }

    pub fn indent(mut self, cols: u16) -> Self {
        self.indent = cols;
        self
    }

    /// Run [`Self::branch_activate`] on branch row `i`.
    fn branch(&mut self, i: usize) -> (Outcome, Option<TreeEvent>) {
        let path = self.rows[i].path.clone();
        match self.branch_activate {
            BranchAction::Toggle => self.toggle(i),
            BranchAction::Activate => {
                self.selected = Some(path.clone());
                (Outcome::Changed, Some(TreeEvent::Activate(path)))
            }
            BranchAction::Both => {
                let (o, ev) = self.toggle(i);
                self.selected = Some(path.clone());
                (
                    o.or(Outcome::Changed),
                    ev.or(Some(TreeEvent::Activate(path))),
                )
            }
        }
    }

    pub fn rows(&self) -> &[FlatRow] {
        &self.rows
    }

    /// Keep `cells` free at the right of every row for owner-drawn columns.
    pub fn reserve_right(mut self, cells: u16) -> Self {
        self.reserve_right = cells;
        self
    }

    /// Show or hide the per-row trailing metadata.
    pub fn show_meta(mut self, on: bool) -> Self {
        self.show_meta = on;
        self
    }

    /// The reserved trailing strip as last drawn; `Rect::ZERO` when there is
    /// none (nothing reserved, or the tree is too narrow to keep a label of
    /// at least [`TreeView::MIN_LABEL`] cells beside it).
    pub fn trailing(&self) -> Rect {
        self.trailing
    }

    /// The narrowest label the reserved strip may leave; below it the strip
    /// is dropped rather than starve the labels.
    pub const MIN_LABEL: u16 = 12;

    pub fn flatten(&mut self) {
        // the focused node is an identity, not a display position: capture
        // it before the rebuild and resolve it afterwards
        let focused: Option<Path> = self.rows.get(self.cursor).map(|r| r.path.clone());
        let before: Option<Path> = self
            .cursor
            .checked_sub(1)
            .and_then(|i| self.rows.get(i))
            .map(|r| r.path.clone());
        let after: Option<Path> = self.rows.get(self.cursor + 1).map(|r| r.path.clone());
        fn matches(n: &TreeNode, q: &str) -> bool {
            n.label.to_lowercase().contains(q) || n.children.iter().any(|c| matches(c, q))
        }
        fn walk(
            nodes: &[TreeNode],
            path: &mut Path,
            depth: usize,
            exp: &std::collections::HashSet<Path>,
            filter: Option<&str>,
            out: &mut Vec<FlatRow>,
        ) {
            for (i, n) in nodes.iter().enumerate() {
                if let Some(q) = filter
                    && !matches(n, q)
                {
                    continue;
                }
                path.push(i);
                let has_children = !n.children.is_empty() || (n.lazy && !n.note);
                // while filtering, folders that contain matches are open
                let expanded = has_children
                    && (exp.contains(path)
                        || filter.is_some_and(|q| {
                            !n.children.is_empty() && !n.label.to_lowercase().contains(q)
                        }));
                out.push(FlatRow {
                    path: path.clone(),
                    depth,
                    label: n.label.clone(),
                    meta: n.meta.clone(),
                    has_children,
                    expanded,
                    glyph: n.glyph,
                    busy: n.busy,
                    note: n.note,
                });
                if expanded {
                    walk(&n.children, path, depth + 1, exp, filter, out);
                }
                path.pop();
            }
        }
        let mut out = Vec::new();
        let filter = self
            .filter
            .as_deref()
            .map(|f| f.trim().to_lowercase())
            .filter(|f| !f.is_empty());
        walk(
            &self.nodes,
            &mut Vec::new(),
            0,
            &self.expanded,
            filter.as_deref(),
            &mut out,
        );
        self.rows = out;
        self.cursor = self
            .relocate(focused.as_deref(), before.as_deref(), after.as_deref())
            .unwrap_or(self.cursor)
            .min(self.rows.len().saturating_sub(1));
        self.scroll.set_content(self.rows.len());
    }

    /// Where the focused node went: itself, else its nearest visible
    /// ancestor, else the surviving neighbour (next, then previous).
    fn relocate(
        &self,
        focused: Option<&[usize]>,
        before: Option<&[usize]>,
        after: Option<&[usize]>,
    ) -> Option<usize> {
        let find = |p: &[usize]| self.rows.iter().position(|r| r.path == p);
        let focused = focused?;
        if let Some(i) = find(focused) {
            return Some(i);
        }
        for k in (1..focused.len()).rev() {
            if let Some(i) = find(&focused[..k]) {
                return Some(i);
            }
        }
        after.and_then(find).or_else(|| before.and_then(find))
    }

    /// The path under the cursor, when the tree has rows.
    pub fn cursor_path(&self) -> Option<&[usize]> {
        self.rows.get(self.cursor).map(|r| r.path.as_slice())
    }

    pub fn node(&self, path: &[usize]) -> Option<&TreeNode> {
        let mut nodes = &self.nodes;
        let mut cur: Option<&TreeNode> = None;
        for &i in path {
            cur = nodes.get(i);
            nodes = &cur?.children;
        }
        cur
    }

    pub fn node_mut(&mut self, path: &[usize]) -> Option<&mut TreeNode> {
        let (last, parents) = path.split_last()?;
        let mut nodes = &mut self.nodes;
        for &i in parents {
            nodes = &mut nodes.get_mut(i)?.children;
        }
        nodes.get_mut(*last)
    }

    /// Deliver lazily fetched children; marks the node loaded and open.
    pub fn set_children(&mut self, path: &[usize], children: Vec<TreeNode>) {
        if let Some(n) = self.node_mut(path) {
            n.children = children;
            n.lazy = false;
            n.busy = false;
        }
        self.expanded.insert(path.to_vec());
        self.flatten();
    }

    pub fn set_busy(&mut self, path: &[usize], busy: bool) {
        if let Some(n) = self.node_mut(path) {
            n.busy = busy;
        }
        self.flatten();
    }

    pub fn set_filter(&mut self, q: Option<&str>) {
        self.filter = q.map(str::to_owned);
        self.flatten();
        self.scroll.jump_start();
        self.cursor = 0;
    }

    /// Move the cursor to a path (expanding ancestors) and select it.
    pub fn reveal(&mut self, path: &[usize]) {
        for k in 1..path.len() {
            self.expanded.insert(path[..k].to_vec());
        }
        self.flatten();
        if let Some(i) = self.rows.iter().position(|r| r.path == path) {
            self.set_cursor(i);
        }
    }

    pub fn row_id(&self, i: usize) -> WidgetId {
        self.id.child(i)
    }

    pub fn toggle_id(&self, i: usize) -> WidgetId {
        self.id.child(i).sub("toggle")
    }

    fn set_cursor(&mut self, i: usize) {
        self.cursor = i.min(self.rows.len().saturating_sub(1));
        self.scroll.ensure_visible(self.cursor);
    }

    pub fn toggle(&mut self, i: usize) -> (Outcome, Option<TreeEvent>) {
        let Some(row) = self.rows.get(i) else {
            return (Outcome::Consumed, None);
        };
        if !row.has_children {
            return (Outcome::Consumed, None);
        }
        let path = row.path.clone();
        if !self.expanded.remove(&path) {
            self.expanded.insert(path.clone());
            let lazy = self
                .node(&path)
                .is_some_and(|n| n.lazy && n.children.is_empty());
            self.flatten();
            if lazy {
                if let Some(n) = self.node_mut(&path) {
                    n.busy = true;
                }
                self.flatten();
                return (Outcome::Changed, Some(TreeEvent::Expand(path)));
            }
            return (Outcome::Changed, None);
        }
        self.flatten();
        (Outcome::Changed, None)
    }

    pub fn expand_all(&mut self) {
        fn walk(nodes: &[TreeNode], path: &mut Path, exp: &mut std::collections::HashSet<Path>) {
            for (i, n) in nodes.iter().enumerate() {
                path.push(i);
                if !n.children.is_empty() {
                    exp.insert(path.clone());
                    walk(&n.children, path, exp);
                }
                path.pop();
            }
        }
        walk(&self.nodes, &mut Vec::new(), &mut self.expanded);
        self.flatten();
    }

    pub fn collapse_all(&mut self) {
        self.expanded.clear();
        self.flatten();
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TreeEvent>) {
        if self.rows.is_empty() {
            return (Outcome::Ignored, None);
        }
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            // an unassigned modified chord never performs a plain action
            return (Outcome::Ignored, None);
        }
        let out = match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.set_cursor(self.cursor.saturating_sub(1));
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.set_cursor(self.cursor + 1);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.set_cursor(self.cursor.saturating_sub(self.scroll.viewport_len.max(1)));
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.set_cursor(self.cursor + self.scroll.viewport_len.max(1));
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.set_cursor(0);
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.set_cursor(usize::MAX);
                Outcome::Changed
            }
            KeyCode::Right | KeyCode::Char('l') => {
                let row = &self.rows[self.cursor];
                if row.has_children && !row.expanded {
                    return self.toggle(self.cursor);
                } else if row.has_children {
                    self.set_cursor(self.cursor + 1);
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            KeyCode::Left | KeyCode::Char('h') => {
                let row = &self.rows[self.cursor];
                if row.has_children && row.expanded {
                    return self.toggle(self.cursor);
                } else if row.depth > 0 {
                    let parent = &row.path[..row.path.len() - 1];
                    if let Some(pi) = self.rows.iter().position(|r| r.path == parent) {
                        self.set_cursor(pi);
                    }
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let row = &self.rows[self.cursor];
                if row.note {
                    Outcome::Consumed
                } else if row.has_children {
                    return self.branch(self.cursor);
                } else {
                    self.selected = Some(row.path.clone());
                    return (
                        Outcome::Changed,
                        Some(TreeEvent::Activate(row.path.clone())),
                    );
                }
            }
            KeyCode::Char('*') => {
                self.expand_all();
                Outcome::Changed
            }
            KeyCode::Char('-') => {
                self.collapse_all();
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        };
        (out, None)
    }

    pub fn on_click_row(&mut self, i: usize) -> (Outcome, Option<TreeEvent>) {
        if i >= self.rows.len() {
            return (Outcome::Consumed, None);
        }
        self.set_cursor(i);
        if self.rows[i].note {
            (Outcome::Consumed, None)
        } else if self.rows[i].has_children {
            if self.row_click_toggles {
                self.branch(i)
            } else {
                (Outcome::Changed, None)
            }
        } else {
            self.selected = Some(self.rows[i].path.clone());
            (
                Outcome::Changed,
                Some(TreeEvent::Activate(self.rows[i].path.clone())),
            )
        }
    }

    pub fn on_click_toggle(&mut self, i: usize) -> (Outcome, Option<TreeEvent>) {
        self.set_cursor(i);
        self.toggle(i)
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        if self.scroll.scroll_by(delta as isize) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// (row, is_toggle) for a widget id.
    pub fn locate(&self, id: WidgetId) -> Option<(usize, bool)> {
        for i in self.scroll.visible_range() {
            if self.row_id(i) == id {
                return Some((i, false));
            }
            if self.toggle_id(i) == id {
                return Some((i, true));
            }
        }
        None
    }

    pub fn owns(&self, id: WidgetId) -> bool {
        id == self.id || id == scrollbar::id_for(self.id) || self.locate(id).is_some()
    }

    fn track(&self) -> Rect {
        if self.track.is_empty() {
            Rect::new(
                self.area.right().saturating_sub(1),
                self.area.y,
                1,
                self.area.height,
            )
        } else {
            self.track
        }
    }

    /// The pointer went down on the scrollbar (or a completed click): a
    /// press on the thumb grabs it, a press on the track jumps to it.
    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        if scrollbar::press(self.track(), pos, &mut self.scroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    /// The pointer dragged along the scrollbar after a press.
    pub fn on_scrollbar_drag(&mut self, pos: Position) -> Outcome {
        if scrollbar::drag(self.track(), pos, &mut self.scroll) {
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.scroll.set_viewport(area.height as usize);
        ctx.control(self.id, area, false);
        ctx.scrollable(self.id, area);
        let has_sb = self.scroll.overflows();
        let full_w = area.width.saturating_sub(if has_sb { 1 } else { 0 });
        // the owner's trailing columns keep their strip while a usable label
        // column remains beside them; labels end before the strip
        let reserve = if self.reserve_right > 0 && full_w >= self.reserve_right + Self::MIN_LABEL {
            self.reserve_right
        } else {
            0
        };
        self.trailing = if reserve > 0 {
            Rect::new(area.x + full_w - reserve, area.y, reserve, area.height)
        } else {
            Rect::ZERO
        };
        let row_w = full_w - reserve;
        // metadata is a column: either every visible row shows it or none does
        let show_meta = self.show_meta
            && self.scroll.visible_range().all(|ri| {
                let row = &self.rows[ri];
                let Some(m) = &row.meta else { return true };
                let need = 1
                    + (row.depth as u16) * self.indent
                    + 2
                    + if row.glyph.is_some() { 2 } else { 0 }
                    + crate::ui::text::width(&row.label) as u16
                    + 2
                    + crate::ui::text::width(m) as u16
                    + 1;
                need <= row_w
            });
        for (i, ri) in self.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let row = &self.rows[ri];
            let rect = Rect::new(area.x, y, row_w, 1);
            // the row (fill, press, click) spans the reserved strip too
            let full = Rect::new(area.x, y, full_w, 1);
            let rid = self.row_id(ri);
            let mut s = ctx.child_state(self.id, rid, ri == self.cursor);
            let tid = self.toggle_id(ri);
            // the expander is part of the row: its hover and press are the row's
            s.hovered |= ctx.interaction.hovered(tid);
            s.pressed |= ctx.interaction.pressed(tid);
            s.focused = focused && ri == self.cursor;
            s.selected = self.selected.as_ref() == Some(&row.path);
            let st = t.row(s, bg);
            fill(buf, full, st);
            buf.set_string(
                rect.x,
                y,
                t.gutter_symbol(s),
                t.gutter(s, st.bg.unwrap_or(bg), false),
            );
            let indent = (row.depth as u16) * self.indent;
            let mut x = rect.x + 1 + indent;
            if x + 2 >= rect.right() {
                continue;
            }
            let glyph = if row.busy {
                crate::widgets::progress::spinner_frame(ctx.interaction.tick)
            } else if row.has_children {
                if row.expanded { "▾" } else { "▸" }
            } else {
                " "
            };
            let gs = if row.busy {
                st.fg(t.accent)
            } else if row.has_children {
                st.fg(t.text_secondary)
            } else {
                st
            };
            buf.set_string(x, y, glyph, gs);
            if row.has_children {
                ctx.clickable(tid, Rect::new(x, y, 2, 1));
            }
            x += 2;
            let meta_w = row
                .meta
                .as_ref()
                .map(|m| crate::ui::text::width(m))
                .unwrap_or(0) as u16;
            let avail = rect.right().saturating_sub(x);
            // hide metadata rather than starve the label
            let meta_w = if !show_meta || avail.saturating_sub(meta_w + 2) < 10 {
                0
            } else {
                meta_w
            };
            let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 1 });
            // the selected node is accent only while the tree has focus;
            // otherwise it is a fact, not a position: primary and bold
            let mut label_style = match (s.selected, focused) {
                (true, true) => st.fg(t.chosen(true)),
                (true, false) => st.fg(t.chosen(false)).add_modifier(Modifier::BOLD),
                _ => st,
            };
            if row.note {
                label_style = st.fg(t.text_muted).remove_modifier(Modifier::BOLD);
            }
            let mut lx = x;
            if let Some(g) = row.glyph {
                buf.set_string(
                    lx,
                    y,
                    g,
                    st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                );
                lx += 2;
            }
            buf.set_string(
                lx,
                y,
                crate::ui::text::fit(&row.label, (lw as usize).saturating_sub((lx - x) as usize)),
                label_style,
            );
            if let Some(m) = &row.meta
                && meta_w > 0
                && meta_w + 4 < avail
            {
                buf.set_string(
                    rect.right().saturating_sub(meta_w + 1),
                    y,
                    m,
                    st.fg(t.text_muted),
                );
            }
            ctx.clickable(rid, full);
            // toggle region registered after row so it wins hit-testing
            if row.has_children {
                ctx.clickable(tid, Rect::new(rect.x + 1 + indent, y, 2, 1));
            }
        }
        // the scroll region also sits above the rows: a wheel burst handled
        // before the next frame (see `runtime::Coalesce`) then routes to the
        // widget, not to a row whose display index it already scrolled away
        ctx.scrollable(self.id, area);
        self.track = Rect::ZERO;
        if has_sb {
            crate::ui::fade::scroll_edges_except(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    area.y,
                    (area.right() - 1).saturating_sub(area.x),
                    area.height,
                ),
                &self.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.cursor)
                    .map(|k| area.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            self.track = sb;
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    #[test]
    fn branch_rows_toggle_or_activate_per_config() {
        let nodes = vec![TreeNode::dir("src", vec![TreeNode::leaf("lib.rs")])];
        let enter = Key {
            code: KeyCode::Enter,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        let mut t = TreeView::new(WidgetId::of("t"), nodes.clone());
        assert_eq!(t.rows().len(), 2);
        t.on_key(&enter);
        assert_eq!(t.rows().len(), 1, "Enter collapses by default");
        let mut a =
            TreeView::new(WidgetId::of("t"), nodes.clone()).branch_activate(BranchAction::Activate);
        let (_, ev) = a.on_key(&enter);
        assert_eq!(ev, Some(TreeEvent::Activate(vec![0])));
        assert_eq!(a.rows().len(), 2, "activating does not collapse");
        let mut quiet = TreeView::new(WidgetId::of("t"), nodes).row_click_toggles(false);
        quiet.on_click_row(0);
        assert_eq!(quiet.rows().len(), 2, "a row click only moves the cursor");
    }
    use ratatui::crossterm::event::KeyModifiers;

    fn key(code: KeyCode) -> Key {
        Key {
            code,
            mods: KeyModifiers::NONE,
        }
    }

    fn tree() -> TreeView {
        TreeView::new(
            WidgetId::of("t"),
            vec![
                TreeNode::lazy("alpha"),
                TreeNode::lazy("beta"),
                TreeNode::dir("gamma", vec![TreeNode::leaf("g1"), TreeNode::leaf("g2")]),
                TreeNode::leaf("delta"),
            ],
        )
    }

    #[test]
    fn delayed_children_above_the_cursor_keep_the_focused_node() {
        let mut t = tree();
        // focus beta (row 1), then alpha's children arrive above it
        t.on_key(&key(KeyCode::Down));
        assert_eq!(t.cursor_path(), Some(&[1][..]));
        t.set_children(
            &[0],
            vec![
                TreeNode::leaf("a1"),
                TreeNode::leaf("a2"),
                TreeNode::leaf("a3"),
            ],
        );
        assert_eq!(t.cursor_path(), Some(&[1][..]), "beta stays focused");
        assert_eq!(t.cursor, 4);
        // a nested delayed load deeper in the tree does not steal focus either
        t.set_children(&[1], vec![TreeNode::lazy("b1")]);
        assert_eq!(t.cursor_path(), Some(&[1][..]));
        t.set_children(&[1, 0], vec![TreeNode::leaf("b1a")]);
        assert_eq!(t.cursor_path(), Some(&[1][..]));
        // selection is distinct from the cursor
        t.on_key(&key(KeyCode::Down));
        t.on_key(&key(KeyCode::Down));
        t.on_key(&key(KeyCode::Enter));
        assert_eq!(t.selected.as_deref(), Some(&[1, 0, 0][..]));
        t.on_key(&key(KeyCode::Up));
        assert_eq!(t.selected.as_deref(), Some(&[1, 0, 0][..]));
        assert_eq!(t.cursor_path(), Some(&[1, 0][..]));
    }

    #[test]
    fn collapse_relocates_to_the_visible_ancestor_and_removal_to_a_neighbour() {
        let mut t = tree();
        t.set_children(&[0], vec![TreeNode::leaf("a1"), TreeNode::leaf("a2")]);
        t.on_key(&key(KeyCode::Down));
        t.on_key(&key(KeyCode::Down));
        assert_eq!(t.cursor_path(), Some(&[0, 1][..]));
        // Left from a leaf climbs to alpha; Left again collapses it in place
        t.on_key(&key(KeyCode::Left));
        assert_eq!(t.cursor_path(), Some(&[0][..]));
        t.on_key(&key(KeyCode::Left));
        assert!(!t.rows[0].expanded);
        assert_eq!(t.cursor_path(), Some(&[0][..]));
        t.toggle(0);
        assert!(t.rows[0].expanded);
        // mouse toggle path: click the fold glyph of alpha while a child is focused
        t.on_key(&key(KeyCode::Down));
        t.on_click_toggle(0);
        assert_eq!(t.cursor_path(), Some(&[0][..]));
        // a child that vanishes on reload: the surviving neighbour keeps focus
        t.toggle(0);
        t.on_key(&key(KeyCode::Down));
        t.on_key(&key(KeyCode::Down));
        assert_eq!(t.cursor_path(), Some(&[0, 1][..]));
        t.set_children(&[0], vec![TreeNode::leaf("a1")]);
        assert_eq!(
            t.cursor_path(),
            Some(&[0][..]),
            "ancestor wins over neighbour"
        );
        // an empty reload is an empty folder, not a crash
        t.set_children(&[0], vec![]);
        assert!(!t.rows[0].has_children);
        assert_eq!(t.cursor_path(), Some(&[0][..]));
    }

    #[test]
    fn filters_reveal_and_reset_deliberately() {
        let mut t = tree();
        t.set_children(&[0], vec![TreeNode::leaf("a1")]);
        t.on_key(&key(KeyCode::End));
        assert_eq!(t.cursor_path(), Some(&[3][..]));
        t.set_filter(Some("g"));
        assert_eq!(t.cursor, 0, "a filter is a new list");
        assert!(
            t.rows
                .iter()
                .all(|r| r.label.contains('g') || r.label == "gamma")
        );
        t.set_filter(None);
        t.reveal(&[2, 1]);
        assert_eq!(t.cursor_path(), Some(&[2, 1][..]));
        t.set_children(&[1], vec![TreeNode::leaf("b1"), TreeNode::leaf("b2")]);
        assert_eq!(
            t.cursor_path(),
            Some(&[2, 1][..]),
            "revealed target survives insertion above"
        );
        // modified chords never navigate
        let before = t.cursor;
        t.on_key(&Key {
            code: KeyCode::Char('j'),
            mods: KeyModifiers::CONTROL,
        });
        assert_eq!(t.cursor, before);
    }
}

#[cfg(test)]
mod scroll_tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;

    fn render(t: &mut TreeView) -> String {
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
        t.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
        (0..6)
            .map(|y| {
                (0..40)
                    .map(|x| buf[(x, y)].symbol().to_owned())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn the_selected_node_is_accent_only_while_the_tree_has_focus() {
        let theme = Theme::junie();
        let nodes = (0..3)
            .map(|i| TreeNode::leaf(&format!("Node {i:02}")))
            .collect();
        let mut tv = TreeView::new(WidgetId::of("t"), nodes);
        tv.selected = tv.rows.get(1).map(|r| r.path.clone());
        assert!(tv.selected.is_some());
        let label_fg = |tv: &mut TreeView, focus: Option<WidgetId>| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let inter = Interaction {
                focus,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
            tv.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
            let x = (0..40)
                .find(|&x| buf[(x, 1)].symbol() == "N")
                .expect("label");
            buf[(x, 1)].fg
        };
        assert_eq!(label_fg(&mut tv, None), theme.text_primary);
        assert_eq!(label_fg(&mut tv, Some(WidgetId::of("t"))), theme.accent);
    }

    #[test]
    fn wheel_moves_the_viewport_and_keeps_the_cursor() {
        let nodes = (0..40)
            .map(|i| TreeNode::leaf(&format!("Node {i:02}")))
            .collect();
        let mut t = TreeView::new(WidgetId::of("t"), nodes);
        let a = render(&mut t);
        assert!(a.contains("Node 00"));
        t.on_wheel(5);
        let b = render(&mut t);
        assert!(!b.contains("Node 00"));
        assert!(b.contains("Node 05"));
        assert_eq!(render(&mut t), b, "render does not reset a wheel scroll");
        assert_eq!(t.cursor, 0);
        let key = Key {
            code: KeyCode::Down,
            mods: ratatui::crossterm::event::KeyModifiers::NONE,
        };
        t.on_key(&key);
        render(&mut t);
        assert!(t.scroll.visible_range().contains(&t.cursor));
    }

    #[test]
    fn reserved_trailing_columns_cut_labels_with_an_ellipsis_and_stay_clear() {
        let nodes = vec![TreeNode::leaf_meta(
            "Parallel Seq Scan on public.orders_archive",
            "0",
        )];
        let mut t = TreeView::new(WidgetId::of("t"), nodes)
            .reserve_right(20)
            .show_meta(false);
        let out = render(&mut t);
        let row: Vec<char> = out.lines().next().unwrap().chars().collect();
        let strip = t.trailing();
        assert_eq!((strip.x, strip.width), (20, 20), "{out}");
        let label: String = row[..20].iter().collect();
        assert!(
            label.trim_end().ends_with('…'),
            "label cut with …: {label:?}"
        );
        assert!(
            row[20..].iter().all(|c| *c == ' '),
            "the strip is left for the owner: {out}"
        );
        // too narrow for a usable label beside the strip: the strip goes
        let mut narrow =
            TreeView::new(WidgetId::of("t"), vec![TreeNode::leaf("x")]).reserve_right(30);
        render(&mut narrow);
        assert_eq!(narrow.trailing(), Rect::ZERO);
    }
}
