use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};

use crate::core::event::{Key, Outcome};
use crate::core::id::{ItemKey, Part, PartRef, WidgetId};
use crate::core::keymap::{Actions, KeyBinding, KeyMap, Nav, NavKeys, lookup, unassigned_chord};
use crate::core::list::{Cursor, KeyStep, Moved, Navigable, RowSource};
use crate::core::scroll::{ScrollPane, Scrollable};
use crate::core::selection::ClickPolicy;
use crate::theme::Tone;
use crate::ui::ctx::RenderCtx;
use crate::ui::list_core::ListCore;
use crate::ui::row::{MarkKind, RowLook, RowUi};

#[derive(Debug, Clone)]
pub struct TreeNode {
    pub label: String,
    pub children: Vec<TreeNode>,
    pub meta: Option<String>,

    pub glyph: Option<std::borrow::Cow<'static, str>>,

    pub glyph_tone: Tone,

    pub label_tone: Option<Tone>,

    pub lazy: bool,

    pub busy: bool,

    pub note: bool,
}

impl TreeNode {
    pub fn leaf(label: &str) -> Self {
        Self {
            label: label.to_owned(),
            children: vec![],
            meta: None,
            glyph: None,
            glyph_tone: Tone::Muted,
            label_tone: None,
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

    pub fn lazy(label: &str) -> Self {
        Self {
            lazy: true,
            ..Self::leaf(label)
        }
    }

    pub fn glyph(mut self, g: impl Into<std::borrow::Cow<'static, str>>) -> Self {
        self.glyph = Some(g.into());
        self
    }

    pub fn glyph_tone(mut self, tone: Tone) -> Self {
        self.glyph_tone = tone;
        self
    }

    pub fn label_tone(mut self, tone: Tone) -> Self {
        self.label_tone = Some(tone);
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

pub type Path = Vec<usize>;

#[derive(Debug, Clone)]
pub struct FlatRow {
    pub path: Path,
    pub depth: usize,
    pub label: String,
    pub meta: Option<String>,
    pub has_children: bool,
    pub expanded: bool,
    pub glyph: Option<std::borrow::Cow<'static, str>>,
    pub glyph_tone: Tone,
    pub label_tone: Option<Tone>,
    pub busy: bool,
    pub note: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeAction {
    Expand(Path),

    Activate(Path),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BranchAction {
    #[default]
    Toggle,

    Activate,

    Both,
}

#[derive(Debug, Clone)]
pub struct TreeView {
    pub id: WidgetId,
    pub nodes: Vec<TreeNode>,
    pub expanded: std::collections::HashSet<Path>,

    pub core: ListCore,
    pub selected: Option<Path>,
    pub area: Rect,
    rows: Vec<FlatRow>,

    pub filter: Option<String>,

    pub reserve_right: u16,

    pub show_meta: bool,

    trailing: Rect,

    pub branch_activate: BranchAction,

    pub indent: u16,

    rows_built: usize,

    pub keymap: KeyMap<TreeCmd>,

    pub divider: bool,

    pub selected_marker: Option<&'static str>,

    pub alt_click_recursive: bool,
}

impl std::ops::Deref for TreeView {
    type Target = ListCore;
    fn deref(&self) -> &ListCore {
        &self.core
    }
}

impl std::ops::DerefMut for TreeView {
    fn deref_mut(&mut self) -> &mut ListCore {
        &mut self.core
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeCmd {
    ExpandAll,

    CollapseAll,
}

impl Actions for TreeCmd {
    fn label(self) -> &'static str {
        match self {
            TreeCmd::ExpandAll => "Expand all",
            TreeCmd::CollapseAll => "Collapse all",
        }
    }

    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::char('*'), TreeCmd::ExpandAll),
            (KeyBinding::char('-'), TreeCmd::CollapseAll),
        ]
    }
}

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
            glyph: n.glyph.clone(),
            glyph_tone: n.glyph_tone,
            label_tone: n.label_tone,
            busy: n.busy,
            note: n.note,
        });
        if expanded {
            walk(&n.children, path, depth + 1, exp, filter, out);
        }
        path.pop();
    }
}

impl TreeView {
    pub fn new(id: WidgetId, nodes: Vec<TreeNode>) -> Self {
        let mut tv = Self {
            id,
            nodes,
            expanded: Default::default(),
            core: ListCore::new(0),
            selected: None,
            area: Rect::ZERO,
            rows: vec![],
            filter: None,
            reserve_right: 0,
            show_meta: true,
            trailing: Rect::ZERO,
            branch_activate: BranchAction::Toggle,
            indent: 2,
            rows_built: 0,
            keymap: TreeCmd::default_keymap(),
            divider: false,
            selected_marker: None,
            alt_click_recursive: true,
        };

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

    pub fn click(mut self, policy: ClickPolicy) -> Self {
        self.core.click = policy;
        self
    }

    pub fn wrap(mut self, on: bool) -> Self {
        self.core.wrap = on;
        self
    }

    pub fn nav(mut self, nav: NavKeys) -> Self {
        self.core.nav = nav;
        self
    }

    pub fn keymap(mut self, keymap: KeyMap<TreeCmd>) -> Self {
        self.keymap = keymap;
        self
    }

    pub fn indent(mut self, cols: u16) -> Self {
        self.indent = cols;
        self
    }

    pub fn type_ahead_timeout(mut self, timeout: std::time::Duration) -> Self {
        self.core.typeahead.timeout = timeout;
        self
    }

    pub fn type_ahead(mut self, on: bool) -> Self {
        self.core.type_ahead = on;
        self
    }

    pub fn type_ahead_precedence(
        mut self,
        precedence: crate::core::list::TypeAheadPrecedence,
    ) -> Self {
        self.core.type_ahead_precedence = Some(precedence);
        self
    }

    pub fn content(mut self, content: crate::ui::content::ContentState) -> Self {
        self.core.content = content;
        self
    }

    pub fn empty(mut self, state: crate::widgets::empty::EmptyState) -> Self {
        self.core.empty = Some(state);
        self
    }

    pub fn skip_when_empty(mut self, on: bool) -> Self {
        self.core.skip_when_empty = on;
        self
    }

    pub fn divider(mut self, on: bool) -> Self {
        self.divider = on;
        self
    }

    pub fn inactive_selection(mut self, look: crate::theme::InactiveSelection) -> Self {
        self.core.inactive_selection = Some(look);
        self
    }

    pub fn selected_marker(mut self, marker: Option<&'static str>) -> Self {
        self.selected_marker = marker;
        self
    }

    pub fn alt_click_recursive(mut self, on: bool) -> Self {
        self.alt_click_recursive = on;
        self
    }

    fn branch(&mut self, i: usize) -> (Outcome, Option<TreeAction>) {
        let path = self.rows[i].path.clone();
        match self.branch_activate {
            BranchAction::Toggle => self.toggle(i),
            BranchAction::Activate => {
                self.selected = Some(path.clone());
                (Outcome::Changed, Some(TreeAction::Activate(path)))
            }
            BranchAction::Both => {
                let (o, ev) = self.toggle(i);
                self.selected = Some(path.clone());
                (
                    o.or(Outcome::Changed),
                    ev.or(Some(TreeAction::Activate(path))),
                )
            }
        }
    }

    pub fn rows(&self) -> &[FlatRow] {
        &self.rows
    }

    pub fn reserve_right(mut self, cells: u16) -> Self {
        self.reserve_right = cells;
        self
    }

    pub fn show_meta(mut self, on: bool) -> Self {
        self.show_meta = on;
        self
    }

    pub fn trailing(&self) -> Rect {
        self.trailing
    }

    pub const MIN_LABEL: u16 = 12;

    pub fn flatten(&mut self) {
        let focused: Option<Path> = self.rows.get(self.core.cursor).map(|r| r.path.clone());
        let before: Option<Path> = self
            .cursor
            .checked_sub(1)
            .and_then(|i| self.rows.get(i))
            .map(|r| r.path.clone());
        let after: Option<Path> = self.rows.get(self.core.cursor + 1).map(|r| r.path.clone());
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
        self.rows_built += out.len();
        self.rows = out;
        self.core.cursor = self
            .relocate(focused.as_deref(), before.as_deref(), after.as_deref())
            .unwrap_or(self.core.cursor)
            .min(self.rows.len().saturating_sub(1));
        self.core.scroll.set_content(self.rows.len());
    }

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

    pub fn cursor_path(&self) -> Option<&[usize]> {
        self.rows.get(self.core.cursor).map(|r| r.path.as_slice())
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
        self.core.scroll.jump_start();
        self.core.cursor = 0;
    }

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
        self.id.part(PartRef::item(Part::ROW, ItemKey::index(i)))
    }

    pub fn toggle_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::ICON, ItemKey::index(i)))
    }

    pub fn row_at(&self, ev: &crate::core::pointer::PointerEvent) -> Option<usize> {
        if !ev.on(self.id) {
            return None;
        }
        ev.index_of(Part::ROW)
            .or_else(|| ev.index_of(Part::ICON))
            .or_else(|| ev.index_of(Part::MARKER))
    }

    pub fn glyph_id(&self, i: usize) -> WidgetId {
        self.id.part(PartRef::item(Part::MARKER, ItemKey::index(i)))
    }

    fn nav_cursor(&self) -> Cursor {
        self.core.cursor_model()
    }

    fn set_cursor(&mut self, i: usize) {
        self.core.cursor = i.min(self.rows.len().saturating_sub(1));
        self.core.scroll.ensure_visible(self.core.cursor);
    }

    pub fn toggle(&mut self, i: usize) -> (Outcome, Option<TreeAction>) {
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
            if lazy && let Some(n) = self.node_mut(&path) {
                n.busy = true;
            }
            self.refold(i);
            if lazy {
                return (Outcome::Changed, Some(TreeAction::Expand(path)));
            }
            return (Outcome::Changed, None);
        }
        self.refold(i);
        (Outcome::Changed, None)
    }

    fn refold(&mut self, i: usize) {
        if self.filter.as_deref().is_some_and(|f| !f.trim().is_empty()) {
            self.flatten();
            return;
        }
        let depth = self.rows[i].depth;

        let end = self.rows[i + 1..]
            .iter()
            .position(|r| r.depth <= depth)
            .map_or(self.rows.len(), |k| i + 1 + k);
        let mut path = self.rows[i].path.clone();
        let Some(node) = self.node(&path) else {
            self.flatten();
            return;
        };
        let has_children = !node.children.is_empty() || (node.lazy && !node.note);
        let open = has_children && self.expanded.contains(&path);
        let busy = node.busy;
        let mut shown = Vec::new();
        if open {
            walk(
                &node.children,
                &mut path,
                depth + 1,
                &self.expanded,
                None,
                &mut shown,
            );
        }
        self.rows_built += shown.len();
        let row = &mut self.rows[i];
        row.expanded = open;
        row.has_children = has_children;
        row.busy = busy;
        let removed = end - (i + 1);
        let added = shown.len();
        self.rows.splice(i + 1..end, shown);

        if self.core.cursor > i {
            self.core.cursor = if self.core.cursor < end {
                if removed > 0 { i } else { self.core.cursor }
            } else {
                self.core.cursor - removed + added
            };
        }
        self.core.cursor = self.core.cursor.min(self.rows.len().saturating_sub(1));
        self.core.scroll.set_content(self.rows.len());
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

    fn all_expanded(&self) -> bool {
        fn walk(
            nodes: &[TreeNode],
            path: &mut Path,
            exp: &std::collections::HashSet<Path>,
        ) -> bool {
            nodes.iter().enumerate().all(|(i, n)| {
                path.push(i);
                let ok =
                    n.children.is_empty() || (exp.contains(path) && walk(&n.children, path, exp));
                path.pop();
                ok
            })
        }
        walk(&self.nodes, &mut Vec::new(), &self.expanded)
    }

    pub fn context_items(&self) -> Vec<(TreeCmd, crate::widgets::menu::MenuItem)> {
        use crate::ui::context::{ContextCmd, command_items};
        let cmds = [
            ContextCmd::new(TreeCmd::ExpandAll)
                .unless(self.all_expanded().then_some("Every branch is open")),
            ContextCmd::new(TreeCmd::CollapseAll)
                .unless(self.expanded.is_empty().then_some("Every branch is closed")),
        ];
        command_items(&self.keymap, &cmds)
    }

    pub fn run(&mut self, cmd: TreeCmd) -> (Outcome, Option<TreeAction>) {
        match cmd {
            TreeCmd::ExpandAll => self.expand_all(),
            TreeCmd::CollapseAll => self.collapse_all(),
        }
        (Outcome::Changed, None)
    }

    pub fn on_key(&mut self, key: &Key) -> (Outcome, Option<TreeAction>) {
        if self.rows.is_empty() {
            return (Outcome::Ignored, None);
        }
        if unassigned_chord(key, &self.keymap) {
            return (Outcome::Ignored, None);
        }
        if let Some(action) = lookup(&self.keymap, key) {
            return self.run(action);
        }
        let rows = &self.rows[..];
        let nav = match self.core.key_step(key, rows) {
            Some((KeyStep::Type(_), moved)) => {
                if let Moved::To(i) = moved {
                    self.set_cursor(i);
                }
                return (moved.outcome(), None);
            }
            Some((KeyStep::Nav(nav), moved)) => (nav, moved),
            None => return self.on_plain_key(key),
        };
        let (nav, moved) = nav;
        let out = match nav {
            Nav::Up | Nav::Down | Nav::PageUp | Nav::PageDown | Nav::Home | Nav::End => {
                if let Some(i) = moved.index() {
                    self.set_cursor(i);
                }
                Outcome::Changed
            }
            Nav::Right => {
                let row = &self.rows[self.core.cursor];
                if row.has_children && !row.expanded {
                    return self.toggle(self.core.cursor);
                } else if row.has_children {
                    self.set_cursor(self.core.cursor + 1);
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            Nav::Left => {
                let row = &self.rows[self.core.cursor];
                if row.has_children && row.expanded {
                    return self.toggle(self.core.cursor);
                } else if row.depth > 0 {
                    let depth = row.depth;
                    if let Some(pi) = self.rows[..self.core.cursor]
                        .iter()
                        .rposition(|r| r.depth < depth)
                    {
                        self.set_cursor(pi);
                    }
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
        };
        (out, None)
    }

    fn on_plain_key(&mut self, key: &Key) -> (Outcome, Option<TreeAction>) {
        let out = match key.code {
            KeyCode::Enter | KeyCode::Char(' ') => {
                let row = &self.rows[self.core.cursor];
                if row.note {
                    Outcome::Consumed
                } else if row.has_children {
                    return self.branch(self.core.cursor);
                } else {
                    self.selected = Some(row.path.clone());
                    return (
                        Outcome::Changed,
                        Some(TreeAction::Activate(row.path.clone())),
                    );
                }
            }
            _ => Outcome::Ignored,
        };
        (out, None)
    }

    pub fn on_click_row(&mut self, i: usize) -> (Outcome, Option<TreeAction>) {
        self.on_click_row_count(i, 1)
    }

    pub fn on_click_row_count(&mut self, i: usize, count: u8) -> (Outcome, Option<TreeAction>) {
        if i >= self.rows.len() {
            return (Outcome::Consumed, None);
        }
        let again = self.core.cursor == i;
        self.core.scroll.pointer_move();
        self.set_cursor(i);
        if self.rows[i].note {
            return (Outcome::Consumed, None);
        }
        if !self.core.click.activates(count.max(1), again) {
            return (Outcome::Changed, None);
        }
        if self.rows[i].has_children {
            self.branch(i)
        } else {
            self.selected = Some(self.rows[i].path.clone());
            (
                Outcome::Changed,
                Some(TreeAction::Activate(self.rows[i].path.clone())),
            )
        }
    }

    pub fn on_click_toggle(&mut self, i: usize) -> (Outcome, Option<TreeAction>) {
        self.core.scroll.pointer_move();
        self.set_cursor(i);
        self.toggle(i)
    }

    pub fn on_click_with(
        &mut self,
        i: usize,
        toggle: bool,
        mods: KeyModifiers,
    ) -> (Outcome, Option<TreeAction>) {
        self.on_click_at(i, toggle, mods, 1)
    }

    pub fn on_click_at(
        &mut self,
        i: usize,
        toggle: bool,
        mods: KeyModifiers,
        count: u8,
    ) -> (Outcome, Option<TreeAction>) {
        if self.alt_click_recursive
            && mods.contains(KeyModifiers::ALT)
            && self.rows.get(i).is_some_and(|r| r.has_children && !r.note)
        {
            self.set_cursor(i);
            return self.toggle_recursive(i);
        }
        if toggle {
            self.on_click_toggle(i)
        } else {
            self.on_click_row_count(i, count)
        }
    }

    pub fn toggle_recursive(&mut self, i: usize) -> (Outcome, Option<TreeAction>) {
        let Some(row) = self.rows.get(i) else {
            return (Outcome::Consumed, None);
        };
        if !row.has_children {
            return (Outcome::Consumed, None);
        }
        let path = row.path.clone();
        let open = !row.expanded;
        fn walk(
            nodes: &[TreeNode],
            path: &mut Path,
            open: bool,
            exp: &mut std::collections::HashSet<Path>,
        ) {
            for (k, n) in nodes.iter().enumerate() {
                path.push(k);
                if !n.children.is_empty() {
                    if open {
                        exp.insert(path.clone());
                    } else {
                        exp.remove(path);
                    }
                    walk(&n.children, path, open, exp);
                }
                path.pop();
            }
        }
        let lazy = self
            .node(&path)
            .is_some_and(|n| n.lazy && n.children.is_empty());
        if let Some(children) = self.node(&path).map(|n| n.children.clone()) {
            walk(&children, &mut path.clone(), open, &mut self.expanded);
        }
        if open {
            self.expanded.insert(path.clone());
            if lazy && let Some(n) = self.node_mut(&path) {
                n.busy = true;
            }
        } else {
            self.expanded.remove(&path);
        }
        self.refold(i);
        if open && lazy {
            return (Outcome::Changed, Some(TreeAction::Expand(path)));
        }
        (Outcome::Changed, None)
    }

    pub fn track_click(mut self, click: crate::core::scroll::TrackClick) -> Self {
        self.core.scroll.track_click = Some(click);
        self
    }

    pub fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, bg: Color) {
        self.core.scroll.begin(ctx.track_click);
        let area = area.intersection(*buf.area());
        if area.is_empty() {
            return;
        }
        self.area = area;
        let t = ctx.theme;
        let focused = ctx.interaction.focused(self.id);
        self.core.scroll.set_viewport(area.height as usize);
        let n = self.rows.len();

        if self.core.is_focus_stop(n, focused) {
            ctx.control(self.id, area, false);
            ctx.scrollable(self.id, area);
        }

        if (!self.core.content.is_ready() || self.core.empty.is_some())
            && self.core.draw_content(n, area, buf, ctx, bg, None)
        {
            return;
        }
        let has_sb = self.core.scroll.overflows();
        let full_w = area
            .width
            .saturating_sub(if has_sb || self.divider { 1 } else { 0 });

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

        let show_meta = self.show_meta
            && self.core.scroll.visible_range().all(|ri| {
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
        for (i, ri) in self.core.scroll.visible_range().enumerate() {
            let y = area.y + i as u16;
            let row = &self.rows[ri];
            let rect = Rect::new(area.x, y, row_w, 1);

            let full = Rect::new(area.x, y, full_w, 1);
            let rid = self.row_id(ri);
            let mut s = ctx.child_state(self.id, rid, ri == self.core.cursor);
            let tid = self.toggle_id(ri);

            s.hovered |= ctx.interaction.hovered(tid);
            s.pressed |= ctx.interaction.pressed(tid);

            let gid = self.glyph_id(ri);
            s.hovered |= ctx.interaction.hovered(gid);
            s.pressed |= ctx.interaction.pressed(gid);
            if row.note {
                s.hovered = false;
                s.pressed = false;
            }
            s.focused = focused && ri == self.core.cursor;
            s.selected = self.selected.as_ref() == Some(&row.path);
            s.inactive_selection = self.core.inactive_selection;
            let rp = RowUi::paint(ctx, buf, full, s, RowLook::Bar, bg);
            let st = rp.style();
            let indent = (row.depth as u16) * self.indent;
            let mut x = rect.x + 1 + indent;

            if !row.note {
                ctx.register_part(self.id, PartRef::item(Part::ROW, ItemKey::index(ri)), full);
            }
            if x + 2 >= rect.right() {
                continue;
            }
            let glyph = if row.busy {
                t.glyphs.spinner_frame(ctx.interaction.tick)
            } else if row.has_children {
                if row.expanded {
                    t.glyphs.disclosure_open
                } else {
                    t.glyphs.disclosure_closed
                }
            } else {
                " "
            };
            let gs = if row.busy {
                st.fg(t.legible_mark(t.accent, st.bg.unwrap_or(bg)))
            } else if row.has_children {
                st.fg(t.text_secondary)
            } else {
                st
            };
            buf.set_string(x, y, glyph, gs);
            if row.has_children {
                ctx.register_part(
                    self.id,
                    PartRef::item(Part::ICON, ItemKey::index(ri)),
                    Rect::new(x, y, 2, 1),
                );
            } else if s.selected
                && !row.busy
                && let Some(m) = self.selected_marker
            {
                rp.mark(ctx, buf, x, y, MarkKind::Glyph(m), focused);
            }
            x += 2;
            let meta_w = row
                .meta
                .as_ref()
                .map(|m| crate::ui::text::width(m))
                .unwrap_or(0) as u16;
            let avail = rect.right().saturating_sub(x);

            let meta_w = if !show_meta || avail.saturating_sub(meta_w + 2) < 10 {
                0
            } else {
                meta_w
            };
            let lw = avail.saturating_sub(if meta_w > 0 { meta_w + 2 } else { 1 });

            let mut label_style = match (s.selected, focused) {
                (true, true) => st.fg(t.chosen(true)),
                (true, false) => st
                    .fg(t.legible(t.chosen_in(s), st.bg.unwrap_or(bg)))
                    .add_modifier(Modifier::BOLD),
                _ => match row.label_tone {
                    Some(tone) => st.fg(t.tone_on(tone, st.bg.unwrap_or(t.canvas))),
                    None => st,
                },
            };
            if row.note {
                label_style = st.fg(t.text_muted).remove_modifier(Modifier::BOLD);
            }
            let mut lx = x;
            if let Some(g) = row.glyph.as_deref() {
                if !row.note {
                    ctx.register_part(
                        self.id,
                        PartRef::item(Part::MARKER, ItemKey::index(ri)),
                        Rect::new(lx, y, 2.min(rect.right().saturating_sub(lx)), 1),
                    );
                }
                buf.set_string(
                    lx,
                    y,
                    g,
                    st.fg(if row.glyph_tone == Tone::Muted {
                        t.text_muted
                    } else {
                        t.tone_on(row.glyph_tone, st.bg.unwrap_or(t.canvas))
                    })
                    .remove_modifier(Modifier::BOLD),
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

            if row.has_children {
                ctx.register_part(
                    self.id,
                    PartRef::item(Part::ICON, ItemKey::index(ri)),
                    Rect::new(rect.x + 1 + indent, y, 2, 1),
                );
            }
        }

        ctx.scrollable(self.id, area);
        if self.divider && !has_sb {
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            self.core
                .scroll
                .draw_divider(sb, buf, ctx, self.id, focused);
        }
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
                &self.core.scroll,
                &self
                    .scroll
                    .visible_range()
                    .position(|i| i == self.core.cursor)
                    .map(|k| area.y + k as u16)
                    .into_iter()
                    .collect::<Vec<_>>(),
            );
            let sb = Rect::new(area.right() - 1, area.y, 1, area.height);
            self.core.scroll.draw_v(sb, buf, ctx, self.id, focused);
        }
    }
}

impl crate::ui::measure::Measure for TreeView {
    fn measure(&self, avail: ratatui::layout::Size, ctx: &RenderCtx) -> ratatui::layout::Size {
        if self.rows.is_empty() || !self.core.content.is_ready() {
            return match &self.core.empty {
                Some(e) if self.rows.is_empty() => e.measure(avail, ctx),
                _ => ratatui::layout::Size::new(0, self.rows.len() as u16),
            };
        }
        let w = |s: &str| crate::ui::text::width(s) as u16;
        let width = self
            .rows
            .iter()
            .map(|row| {
                let lead = 1 + (row.depth as u16) * self.indent + 2;
                let label = if row.glyph.is_some() { 2 } else { 0 } + w(&row.label);
                let rest = match row.meta.as_deref().filter(|_| self.show_meta) {
                    Some(m) => label.max(10) + 2 + w(m) + 1,
                    None => label + 1,
                };
                lead + rest
            })
            .max()
            .unwrap_or(0);
        ratatui::layout::Size::new(
            width + u16::from(self.divider) + self.reserve_right,
            self.rows.len() as u16,
        )
    }
}

impl Scrollable for TreeView {
    fn pane(&mut self) -> &mut ScrollPane {
        &mut self.core.scroll
    }
}

impl RowSource for [FlatRow] {
    fn len(&self) -> usize {
        <[FlatRow]>::len(self)
    }
    fn enabled(&self, i: usize) -> bool {
        !self[i].note
    }
    fn label(&self, i: usize) -> &str {
        &self[i].label
    }
}

impl RowSource for TreeView {
    fn len(&self) -> usize {
        self.rows.len()
    }
    fn enabled(&self, i: usize) -> bool {
        self.rows[..].enabled(i)
    }
    fn label(&self, i: usize) -> &str {
        &self.rows[i].label
    }
}

impl Navigable for TreeView {
    fn cursor(&self) -> Cursor {
        self.nav_cursor()
    }
}

const SPECIMEN_ROWS: usize = 4;

impl crate::ui::specimen::Specimen for TreeView {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(
            state,
            Rest | Hover
                | Pressed
                | Focused
                | FocusedHover
                | Selected
                | Busy
                | Loading
                | Error
                | Overflow
                | Narrow
        )
    }

    fn specimen_height(&self, state: crate::ui::specimen::SpecimenState) -> u16 {
        use crate::ui::specimen::SpecimenState::*;
        if matches!(state, Loading | Error) {
            return 1;
        }
        self.rows.len().clamp(1, SPECIMEN_ROWS) as u16
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState::*;
        let mut t = self.clone();
        t.flatten();
        t.cursor = 1.min(t.rows.len().saturating_sub(1));
        let path = t.rows.get(t.cursor).map(|r| r.path.clone());
        match (state, path) {
            (Selected, Some(p)) => t.selected = Some(p),
            (Overflow, Some(p)) => {
                if let Some(n) = t.node_mut(&p) {
                    n.label = format!("{} and a label far longer than its row", n.label);
                }
                t.flatten();
            }
            (Busy, _) => {
                if let Some(p) = t
                    .rows
                    .iter()
                    .find(|r| r.has_children)
                    .map(|r| r.path.clone())
                {
                    t.set_busy(&p, true);
                }
            }
            (Loading | Error, _) => t.core.content = crate::ui::content::specimen(state),
            _ => {}
        }
        let v = state.visual();
        if v.hovered && t.has_rows(t.rows.len()) {
            ctx.interaction.hover = Some(t.row_id(t.cursor));
            if v.pressed {
                ctx.interaction.pressed = Some(t.row_id(t.cursor));
            }
        }
        crate::ui::ctx::fill(buf, area, ratatui::style::Style::new().bg(bg));
        t.render(area, buf, ctx, bg);
    }
}

impl crate::core::pointer::MouseTarget for TreeView {
    type Action = TreeAction;

    fn mouse_id(&self) -> WidgetId {
        self.id
    }

    fn pointer(
        &mut self,
        ev: &crate::core::pointer::PointerEvent,
    ) -> crate::core::response::Response<TreeAction> {
        use crate::core::intent::Phase;
        if let Some(r) = crate::core::pointer::scroll_input(self, ev) {
            return r.lift();
        }
        if !ev.is(Phase::Click) {
            return crate::core::response::Response::ignored();
        }

        let hit = match (ev.index_of(Part::ICON), ev.index_of(Part::ROW)) {
            (Some(row), _) => Some((row, true)),
            (None, Some(row)) => Some((row, false)),
            _ => ev.index_of(Part::MARKER).map(|row| (row, false)),
        };
        match hit {
            Some((row, toggle)) => self.on_click_at(row, toggle, ev.mods(), ev.count()).into(),
            None => crate::core::response::Response::ignored(),
        }
    }
}

#[cfg(test)]
mod identity_tests {
    use super::*;

    fn big(folders: usize, leaves: usize) -> TreeView {
        let dirs = (0..folders)
            .map(|d| {
                let mut kids: Vec<TreeNode> = (0..leaves)
                    .map(|l| TreeNode::leaf(&format!("leaf {d}.{l}")))
                    .collect();
                kids.push(TreeNode::dir("sub", vec![TreeNode::leaf("deep")]));
                TreeNode::dir(&format!("folder {d}"), kids)
            })
            .collect();
        TreeView::new(WidgetId::of("big"), dirs)
    }

    fn shape(t: &TreeView) -> Vec<(Path, usize, bool)> {
        t.rows()
            .iter()
            .map(|r| (r.path.clone(), r.depth, r.expanded))
            .collect()
    }

    #[test]
    fn a_toggle_builds_only_the_rows_it_reveals() {
        let mut t = big(1_000, 100);
        let per = 102;
        assert_eq!(t.rows().len(), 1_000 * per);
        let folder = 500 * per;

        t.toggle(folder + 101);
        t.cursor = folder + 40;
        let built = t.rows_built;
        t.toggle(folder);
        assert_eq!(t.rows_built - built, 0, "a collapse builds no rows");
        assert_eq!(t.cursor, folder, "the cursor moves to the folder");
        let mut full = t.clone();
        full.flatten();
        assert_eq!(shape(&t), shape(&full));
        t.cursor = folder + 1_000;
        let built = t.rows_built;
        t.toggle(folder);
        assert_eq!(
            t.rows_built - built,
            102,
            "an expand builds its 100 leaves, the sub-folder and its child"
        );
        assert_eq!(t.cursor, folder + 1_000 + 102, "the cursor keeps its node");
        let mut full = t.clone();
        full.flatten();
        assert_eq!(shape(&t), shape(&full));
        assert_eq!(t.cursor, full.cursor);
        assert_eq!(t.scroll.content_len, t.rows().len());
    }

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
        assert_eq!(ev, Some(TreeAction::Activate(vec![0])));
        assert_eq!(a.rows().len(), 2, "activating does not collapse");

        let mut data = TreeView::new(WidgetId::of("t"), nodes.clone());
        data.on_click_row(0);
        data.on_click_row(0);
        assert_eq!(data.rows().len(), 2, "a row click only moves the cursor");
        data.on_click_row_count(0, 2);
        assert_eq!(data.rows().len(), 1, "a double click toggles the branch");
        data.on_click_toggle(0);
        assert_eq!(data.rows().len(), 2, "the disclosure toggles on one click");
        let mut leaf = TreeView::new(WidgetId::of("t"), nodes.clone());
        assert_eq!(leaf.on_click_row(1).1, None, "a leaf click selects");
        assert_eq!(
            leaf.on_click_row_count(1, 2).1,
            Some(TreeAction::Activate(vec![0, 0])),
            "a leaf double click activates"
        );

        let mut nav = TreeView::new(WidgetId::of("t"), nodes).click(ClickPolicy::NAVIGATION);
        assert_eq!(
            nav.on_click_row(1).1,
            Some(TreeAction::Activate(vec![0, 0]))
        );
        nav.on_click_row(0);
        assert_eq!(nav.rows().len(), 1, "a branch click toggles");
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

        t.set_children(&[1], vec![TreeNode::lazy("b1")]);
        assert_eq!(t.cursor_path(), Some(&[1][..]));
        t.set_children(&[1, 0], vec![TreeNode::leaf("b1a")]);
        assert_eq!(t.cursor_path(), Some(&[1][..]));

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

        t.on_key(&key(KeyCode::Left));
        assert_eq!(t.cursor_path(), Some(&[0][..]));
        t.on_key(&key(KeyCode::Left));
        assert!(!t.rows[0].expanded);
        assert_eq!(t.cursor_path(), Some(&[0][..]));
        t.toggle(0);
        assert!(t.rows[0].expanded);

        t.on_key(&key(KeyCode::Down));
        t.on_click_toggle(0);
        assert_eq!(t.cursor_path(), Some(&[0][..]));

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
    use crate::core::hit::Axis;
    use crate::core::hit::HitRegistry;
    use crate::theme::Theme;
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    #[test]
    fn a_busy_nodes_spinner_reads_on_its_row_at_every_level() {
        use crate::theme::{ColorLevel, contrast_ratio};
        for level in ColorLevel::ALL {
            let theme = Theme::for_level(level);
            let mut node = TreeNode::leaf("schema");
            node.busy = true;
            let mut t = TreeView::new(WidgetId::of("t"), vec![node]);
            let rid = t.row_id(0);
            for hover in [None, Some(rid)] {
                let mut hits = HitRegistry::default();
                let mut ring = FocusRing::default();
                let inter = Interaction {
                    hover,
                    ..Default::default()
                };
                let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
                let mut buf = Buffer::empty(Rect::new(0, 0, 30, 2));
                t.render(Rect::new(0, 0, 30, 2), &mut buf, &mut ctx, theme.canvas);
                let spin = theme.glyphs.spinner_frame(ctx.interaction.tick);
                let c = (0..30u16)
                    .map(|x| &buf[(x, 0)])
                    .find(|c| c.symbol() == spin)
                    .unwrap_or_else(|| panic!("{level:?}: no spinner"));
                let r = contrast_ratio(c.fg, c.bg).unwrap();
                assert!(r >= 3.0, "{level:?} hover {}: {r:.2}", hover.is_some());
            }
        }
    }

    #[test]
    fn a_tree_at_its_measure_shows_every_label_and_meta() {
        use crate::ui::measure::Measure;
        let mut api = TreeNode::dir("api", vec![TreeNode::leaf_meta("handlers", "12 files")]);
        api.meta = Some("service".into());
        let mut t = TreeView::new(
            WidgetId::of("t"),
            vec![
                api,
                TreeNode::leaf_meta("a rather long name for a leaf", "x"),
            ],
        );
        t.expand_all();
        let theme = Theme::junie();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let size = t.measure(ratatui::layout::Size::new(10, 2), &ctx);
        assert_eq!(size.height, 3);
        let area = Rect::new(0, 0, size.width, size.height);
        let mut buf = Buffer::empty(area);
        t.render(area, &mut buf, &mut ctx, theme.canvas);
        let all: String = (0..size.height)
            .map(|y| {
                (0..size.width)
                    .map(|x| buf[(x, y)].symbol())
                    .collect::<String>()
                    + "\n"
            })
            .collect();
        for part in [
            "api",
            "service",
            "handlers",
            "12 files",
            "a rather long name for a leaf",
        ] {
            assert!(all.contains(part), "{part:?} in\n{all}");
        }
    }

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
    fn content_states_replace_the_rows_and_an_empty_tree_is_skipped() {
        use crate::ui::content::ContentState;
        use crate::widgets::empty::EmptyState;
        let stop = |t: &mut TreeView| {
            let theme = Theme::junie();
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
            t.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
            ring.contains(t.id)
        };
        let mut t = TreeView::new(WidgetId::of("t"), vec![TreeNode::leaf("alpha")]);
        assert!(stop(&mut t));
        t.content = ContentState::loading("Loading schema");
        let s = render(&mut t);
        assert!(s.contains("Loading schema") && !s.contains("alpha"), "{s}");
        assert!(!stop(&mut t), "loading: no focus stop");
        t.content = ContentState::error(EmptyState::new("Could not connect"));
        assert!(render(&mut t).contains("! Could not connect"));
        let mut empty = TreeView::new(WidgetId::of("e"), vec![]);
        assert!(!stop(&mut empty), "K37: an empty tree is skipped");
        let mut empty = TreeView::new(WidgetId::of("e"), vec![])
            .empty(EmptyState::new("No tables"))
            .skip_when_empty(false);
        assert!(render(&mut empty).contains("No tables"));
        assert!(stop(&mut empty), "knob: stays a stop");
    }

    #[test]
    fn glyph_and_label_tones_colour_a_row_and_default_to_muted_and_ink() {
        let theme = Theme::junie();
        let nodes = vec![
            TreeNode::leaf("Plain").glyph("·"),
            TreeNode::leaf("Prod")
                .glyph("◆")
                .glyph_tone(Tone::Danger)
                .label_tone(Tone::Warning),
        ];
        let mut tv = TreeView::new(WidgetId::of("t"), nodes);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
        let mut buf = Buffer::empty(Rect::new(0, 0, 40, 6));
        tv.render(Rect::new(0, 0, 40, 6), &mut buf, &mut ctx, theme.canvas);
        let first = |y: u16, s: &str| {
            (0..40)
                .find(|&x| buf[(x, y)].symbol() == s)
                .map(|x| buf[(x, y)].clone())
                .unwrap()
        };
        assert_eq!(first(0, "·").fg, theme.text_muted, "default glyph");
        let label = first(0, "P").fg;
        let g = first(1, "◆");
        assert_eq!(g.fg, theme.tone_on(Tone::Danger, g.bg), "glyph tone");
        let l = first(1, "P");
        assert_eq!(l.fg, theme.tone_on(Tone::Warning, l.bg), "label tone");
        assert_ne!(label, l.fg, "the default label keeps the row's ink");
    }

    #[test]
    fn the_selected_node_is_accent_only_while_the_tree_has_focus() {
        let theme = Theme::junie().with_inactive_selection(crate::theme::InactiveSelection::None);
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
    fn arrows_and_ends_step_over_notes() {
        use ratatui::crossterm::event::KeyModifiers;
        let key = |code| Key {
            code,
            mods: KeyModifiers::NONE,
        };
        let nodes = vec![
            TreeNode::dir("Empty", vec![TreeNode::note("nothing here")]),
            TreeNode::leaf("Tables"),
            TreeNode::note("end of list"),
        ];
        let mut tv = TreeView::new(WidgetId::of("t"), nodes);
        tv.expand_all();
        let labels: Vec<&str> = tv.rows().iter().map(|r| r.label.as_str()).collect();
        assert_eq!(labels, ["Empty", "nothing here", "Tables", "end of list"]);
        tv.on_key(&key(KeyCode::Down));
        assert_eq!(tv.cursor, 2, "Down passes over the note");
        tv.on_key(&key(KeyCode::Down));
        assert_eq!(tv.cursor, 2, "the trailing note is never reached");
        tv.on_key(&key(KeyCode::End));
        assert_eq!(tv.cursor, 2, "End stops on the last selectable row");
        tv.on_key(&key(KeyCode::Up));
        assert_eq!(tv.cursor, 0);
    }

    #[test]
    fn type_ahead_jumps_to_the_next_matching_row_when_on() {
        use ratatui::crossterm::event::KeyModifiers;
        let key = |c| Key {
            code: KeyCode::Char(c),
            mods: KeyModifiers::NONE,
        };
        let nodes = vec![
            TreeNode::dir("Tables", vec![TreeNode::leaf("Data grid")]),
            TreeNode::dir("Overlays", vec![TreeNode::leaf("Dialogs")]),
        ];
        let mut tv = TreeView::new(WidgetId::of("t"), nodes.clone()).type_ahead(true);
        assert_eq!(tv.on_key(&key('o')).0, Outcome::Changed);
        assert_eq!(tv.cursor_path(), Some(&[1][..]));
        tv.typeahead.reset();
        tv.on_key(&key('d'));
        assert_eq!(
            tv.cursor_path(),
            Some(&[1, 0][..]),
            "the next match after the cursor"
        );
        assert_eq!(
            tv.on_key(&key('z')).0,
            Outcome::Consumed,
            "no match: consumed, never half-typed to the owner (plan §3)"
        );
        let mut on = TreeView::new(WidgetId::of("t"), nodes.clone());
        assert_eq!(
            on.on_key(&key('o')).0,
            Outcome::Changed,
            "on by default (Desktop profile, K34)"
        );
        let mut off = TreeView::new(WidgetId::of("t"), nodes).type_ahead(false);
        assert_eq!(
            off.on_key(&key('o')).0,
            Outcome::Ignored,
            "off: the owner's"
        );
        assert_eq!(off.cursor, 0);
    }

    #[test]
    fn a_divider_keeps_the_last_column_and_the_marker_names_the_selected_leaf() {
        let theme = Theme::junie().with_inactive_selection(crate::theme::InactiveSelection::None);
        let nodes = vec![TreeNode::dir("Group", vec![TreeNode::leaf("Page")])];
        let mut tv = TreeView::new(WidgetId::of("t"), nodes)
            .divider(true)
            .selected_marker(Some("›"));
        tv.selected = Some(vec![0, 0]);
        let draw = |tv: &mut TreeView, focus: Option<WidgetId>| {
            let mut hits = HitRegistry::default();
            let mut ring = FocusRing::default();
            let inter = Interaction {
                focus,
                ..Default::default()
            };
            let mut ctx = RenderCtx::new(&theme, inter, &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 20, 4));
            tv.render(Rect::new(0, 0, 20, 4), &mut buf, &mut ctx, theme.canvas);
            buf
        };
        let buf = draw(&mut tv, None);
        for y in 0..4 {
            assert_eq!(
                buf[(19, y)].symbol(),
                theme.glyphs.divider,
                "row {y}: the divider"
            );
        }
        let x = (0..19)
            .find(|&x| buf[(x, 1)].symbol() == "›")
            .expect("marker on the selected leaf");
        assert_eq!(buf[(x, 1)].fg, theme.text_primary, "neutral without focus");
        let buf = draw(&mut tv, Some(WidgetId::of("t")));
        assert_eq!(buf[(x, 1)].fg, theme.accent, "accent with focus");
        assert!(
            (0..19).all(|x| buf[(x, 0)].symbol() != "›"),
            "a group row keeps its disclosure"
        );
    }

    #[test]
    fn wheel_moves_the_viewport_and_keeps_the_cursor() {
        let nodes = (0..40)
            .map(|i| TreeNode::leaf(&format!("Node {i:02}")))
            .collect();
        let mut t = TreeView::new(WidgetId::of("t"), nodes);
        let a = render(&mut t);
        assert!(a.contains("Node 00"));
        t.wheel(Axis::Vertical, 5).outcome();
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

        let mut narrow =
            TreeView::new(WidgetId::of("t"), vec![TreeNode::leaf("x")]).reserve_right(30);
        render(&mut narrow);
        assert_eq!(narrow.trailing(), Rect::ZERO);
    }

    #[test]
    fn deep_rows_stay_clickable_and_notes_are_inert() {
        let theme = Theme::junie();

        let mut deep = TreeNode::leaf("leaf");
        for k in 0..12 {
            deep = TreeNode::dir(&format!("d{k}"), vec![deep]);
        }
        let mut t = TreeView::new(
            WidgetId::of("t"),
            vec![deep, TreeNode::note("nothing here")],
        );
        t.expand_all();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let area = Rect::new(0, 0, 16, 20);
        let mut buf = Buffer::empty(area);
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            t.render(area, &mut buf, &mut ctx, theme.canvas);
        }
        let deepest = t.rows().iter().position(|r| r.label == "leaf").unwrap();
        assert!(16 <= 1 + t.rows()[deepest].depth as u16 * t.indent + 2);
        assert_eq!(
            hits.hit(Position::new(0, deepest as u16)),
            Some(t.row_id(deepest)),
            "a row indented past the label column is still a click target"
        );
        let note = t.rows().iter().position(|r| r.note).unwrap();
        assert_ne!(
            hits.hit(Position::new(5, note as u16)),
            Some(t.row_id(note)),
            "a note row is not a control"
        );

        let rest = buf[(5, note as u16)].style();
        let mut hits = HitRegistry::default();
        let mut buf = Buffer::empty(area);
        let hover = Interaction {
            hover: Some(t.row_id(note)),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&theme, hover, &mut hits, &mut ring);
        t.render(area, &mut buf, &mut ctx, theme.canvas);
        assert_eq!(buf[(5, note as u16)].style(), rest);
    }

    #[test]
    fn alt_click_opens_and_closes_the_whole_subtree() {
        let mut t = TreeView::new(
            WidgetId::of("t"),
            vec![TreeNode::dir(
                "root",
                vec![TreeNode::dir(
                    "a",
                    vec![TreeNode::dir("b", vec![TreeNode::leaf("c")])],
                )],
            )],
        );

        t.on_click_toggle(0);
        assert_eq!(t.rows().len(), 1);
        t.on_click_with(0, true, KeyModifiers::ALT);
        let labels: Vec<_> = t.rows().iter().map(|r| r.label.clone()).collect();
        assert_eq!(labels, ["root", "a", "b", "c"]);

        t.on_click_with(0, false, KeyModifiers::ALT);
        assert_eq!(t.rows().len(), 1);
        t.on_click_toggle(0);
        assert_eq!(t.rows().len(), 2, "descendants were closed too");

        let mut t = t.alt_click_recursive(false);
        t.on_click_toggle(0);
        t.on_click_with(0, true, KeyModifiers::ALT);
        assert_eq!(t.rows().len(), 2);
    }

    #[test]
    fn a_right_click_offers_expand_and_collapse_all_from_the_key_map() {
        use crate::core::intent::Phase;
        use crate::core::pointer::PointerEvent;
        use crate::ui::context::ContextConfig;
        use crate::widgets::menu::MenuAction;
        use ratatui::crossterm::event::KeyModifiers;
        let theme = Theme::junie();
        let nodes = vec![
            TreeNode::dir("a", vec![TreeNode::dir("a1", vec![TreeNode::leaf("x")])]),
            TreeNode::dir("b", vec![TreeNode::leaf("b1")]),
        ];
        let mut t = TreeView::new(WidgetId::of("t"), nodes);
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        {
            let mut ctx = RenderCtx::new(&theme, Interaction::default(), &mut hits, &mut ring);
            let mut buf = Buffer::empty(Rect::new(0, 0, 30, 8));
            t.render(Rect::new(0, 0, 30, 8), &mut buf, &mut ctx, theme.canvas);
        }
        let pos = Position::new(4, 1);
        let ev = PointerEvent::new(
            &hits,
            Phase::Secondary,
            hits.hit(pos).expect("a row"),
            pos,
            1,
            KeyModifiers::NONE,
        );
        let req = ContextConfig::DEFAULT
            .on_secondary(&mut t, &ev)
            .expect("a row has a menu");
        assert_eq!(t.cursor, 1, "the menu is about the clicked row");
        let row = |t: &TreeView, c: TreeCmd| {
            let items = t.context_items();
            let (_, i) = items.into_iter().find(|(x, _)| *x == c).expect("listed");
            (
                i.item.label.clone(),
                i.shortcut.clone(),
                i.item.reason.clone(),
            )
        };
        assert_eq!(
            row(&t, TreeCmd::ExpandAll),
            ("Expand all".into(), Some("*".into()), None)
        );
        assert_eq!(
            row(&t, TreeCmd::CollapseAll),
            ("Collapse all".into(), Some("-".into()), None)
        );

        let items = t.context_items();
        let mut menu = req.menu(
            WidgetId::of("t.menu"),
            items.iter().map(|(_, i)| i.clone()).collect(),
        );
        assert_eq!(menu.on_click(menu.row_id(0)), Some(MenuAction::Chosen(0)));
        t.run(items[0].0);
        assert_eq!(t.rows().len(), 5, "every branch open");
        assert_eq!(
            row(&t, TreeCmd::ExpandAll).2.as_deref(),
            Some("Every branch is open")
        );
        t.run(TreeCmd::CollapseAll);
        assert_eq!(t.rows().len(), 2);
        assert_eq!(
            row(&t, TreeCmd::CollapseAll).2.as_deref(),
            Some("Every branch is closed")
        );

        t.keymap = vec![(KeyBinding::char('e'), TreeCmd::ExpandAll)];
        assert_eq!(row(&t, TreeCmd::ExpandAll).1.as_deref(), Some("e"));
        assert_eq!(row(&t, TreeCmd::CollapseAll).1, None);
    }
}
