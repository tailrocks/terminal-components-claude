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

#[derive(Debug, Clone)]
pub struct TreeView {
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
            rows: vec![],
            filter: None,
        };
        // expand the first level by default
        for i in 0..tv.nodes.len() {
            tv.expanded.insert(vec![i]);
        }
        tv.flatten();
        tv
    }

    pub fn rows(&self) -> &[FlatRow] {
        &self.rows
    }

    pub fn flatten(&mut self) {
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
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
        self.scroll.set_content(self.rows.len());
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
                    return self.toggle(self.cursor);
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
            self.toggle(i)
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
        self.scroll.scroll_by(delta as isize);
        Outcome::Changed
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

    pub fn on_scrollbar(&mut self, pos: Position) -> Outcome {
        let track = Rect::new(
            self.area.right().saturating_sub(1),
            self.area.y,
            1,
            self.area.height,
        );
        self.scroll
            .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
        Outcome::Changed
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
        let row_w = area.width.saturating_sub(if has_sb { 1 } else { 0 });
        for (i, ri) in self.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let row = &self.rows[ri];
            let rect = Rect::new(area.x, y, row_w, 1);
            let rid = self.row_id(ri);
            let mut s = ctx.state(rid);
            let tid = self.toggle_id(ri);
            if ctx.interaction.hovered(tid) {
                s.hovered = true;
            }
            s.focused = focused && ri == self.cursor;
            s.selected = self.selected.as_ref() == Some(&row.path);
            let st = t.row(s, bg);
            fill(buf, rect, st);
            buf.set_string(rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            let indent = (row.depth * 2) as u16;
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
            let meta_w = if avail.saturating_sub(meta_w + 2) < 10 {
                0
            } else {
                meta_w
            };
            let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 1 });
            let mut label_style = if s.selected { st.fg(t.accent) } else { st };
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
            ctx.clickable(rid, rect);
            // toggle region registered after row so it wins hit-testing
            if row.has_children {
                ctx.clickable(tid, Rect::new(rect.x + 1 + indent, y, 2, 1));
            }
        }
        if has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            scrollbar::render_vertical(sb, buf, ctx, self.id, &self.scroll, focused);
        }
    }
}
