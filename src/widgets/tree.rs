use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::scroll::ScrollState;
use crate::data::TreeNode;
use crate::ui::ctx::{RenderCtx, fill};
use crate::widgets::scrollbar;

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
        fn walk(
            nodes: &[TreeNode],
            path: &mut Path,
            depth: usize,
            exp: &std::collections::HashSet<Path>,
            out: &mut Vec<FlatRow>,
        ) {
            for (i, n) in nodes.iter().enumerate() {
                path.push(i);
                let expanded = exp.contains(path);
                out.push(FlatRow {
                    path: path.clone(),
                    depth,
                    label: n.label.clone(),
                    meta: n.meta.clone(),
                    has_children: !n.children.is_empty(),
                    expanded,
                });
                if expanded {
                    walk(&n.children, path, depth + 1, exp, out);
                }
                path.pop();
            }
        }
        let mut out = Vec::new();
        walk(&self.nodes, &mut Vec::new(), 0, &self.expanded, &mut out);
        self.rows = out;
        self.cursor = self.cursor.min(self.rows.len().saturating_sub(1));
        self.scroll.set_content(self.rows.len());
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

    pub fn toggle(&mut self, i: usize) -> Outcome {
        let Some(row) = self.rows.get(i) else {
            return Outcome::Consumed;
        };
        if !row.has_children {
            return Outcome::Consumed;
        }
        let path = row.path.clone();
        if !self.expanded.remove(&path) {
            self.expanded.insert(path);
        }
        self.flatten();
        Outcome::Changed
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

    pub fn on_key(&mut self, key: &Key) -> Outcome {
        if self.rows.is_empty() {
            return Outcome::Ignored;
        }
        match key.code {
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
                    self.toggle(self.cursor)
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
                    self.toggle(self.cursor)
                } else if row.depth > 0 {
                    // go to parent
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
                if row.has_children {
                    self.toggle(self.cursor)
                } else {
                    self.selected = Some(row.path.clone());
                    Outcome::Changed
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
        }
    }

    pub fn on_click_row(&mut self, i: usize) -> Outcome {
        if i >= self.rows.len() {
            return Outcome::Consumed;
        }
        self.set_cursor(i);
        if self.rows[i].has_children {
            self.toggle(i)
        } else {
            self.selected = Some(self.rows[i].path.clone());
            Outcome::Changed
        }
    }

    pub fn on_click_toggle(&mut self, i: usize) -> Outcome {
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
            let glyph = if row.has_children {
                if row.expanded { "▾" } else { "▸" }
            } else {
                " "
            };
            let gs = if row.has_children {
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
            let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 1 });
            let label_style = if s.selected { st.fg(t.accent) } else { st };
            buf.set_string(
                x,
                y,
                crate::ui::text::fit(&row.label, lw as usize),
                label_style,
            );
            if let Some(m) = &row.meta
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
