//! A tree of panes split side by side and stacked (S26): tmux's or an
//! editor's window layout as a library type.
//!
//! [`PaneNode`] is the binary split tree (every split holds a [`Split`],
//! so nested splits give any number of panes); [`PaneTree`] adds the
//! focused pane, zoom, keyboard focus moves and resize chords, the seams
//! as draggable [`Splitter`]s, and the pane frames: rounded borders that
//! mark the focused pane (strong border, bright title) and the hovered one
//! (brighter title). Zoom maximises the focused pane through
//! [`Split::toggle_max`] on every split above it, so the zoomed layout is
//! the same split arithmetic as any other.
//!
//! The owner draws each pane's content into the inner rect
//! [`PaneTree::render`] hands back, and offers keys and pointer events to
//! [`PaneTree::on_key`] and [`PaneTree::on_pointer`] first. A refused
//! command ("unzoom to resize") comes back as a [`PaneRefusal`] whose
//! message the owner shows in its status line.

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::{Margin, Rect, Size};
use ratatui::style::Color;

use crate::core::event::{Key, Outcome};
use crate::core::id::WidgetId;
use crate::core::intent::Phase;
use crate::core::keymap::{Actions, KeyBinding, KeyMap, hint_key, lookup};
use crate::core::pointer::PointerEvent;
use crate::ui::ctx::RenderCtx;
use crate::ui::layout::{Maximized, Split, SplitDir, SplitKeys};
use crate::widgets::keyhint::{Hint, hint};
use crate::widgets::splitter::Splitter;

/// A pane's id: the owner's key for the pane's content.
pub type PaneId = u64;

/// A node of the split tree: a pane, or a split of two subtrees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PaneNode {
    Leaf(PaneId),
    Split {
        dir: SplitDir,
        split: Split,
        first: Box<PaneNode>,
        second: Box<PaneNode>,
    },
}

impl PaneNode {
    /// The panes, first to last (left to right, top to bottom).
    pub fn leaves(&self) -> Vec<PaneId> {
        match self {
            PaneNode::Leaf(id) => vec![*id],
            PaneNode::Split { first, second, .. } => {
                let mut v = first.leaves();
                v.extend(second.leaves());
                v
            }
        }
    }

    /// Geometry of every pane and every seam laid out in `area`, with
    /// `gap` cells between the panes of a split.
    pub fn layout(
        &self,
        area: Rect,
        gap: u16,
        out: &mut Vec<(PaneId, Rect)>,
        seams: &mut Vec<Seam>,
        path: &mut Vec<u8>,
    ) {
        match self {
            PaneNode::Leaf(id) => out.push((*id, area)),
            PaneNode::Split {
                dir,
                split,
                first,
                second,
            } => {
                let (a, b) = split.layout(*dir, area, gap);
                let handle = split.handle(*dir, area, gap);
                seams.push(Seam {
                    path: path.clone(),
                    dir: *dir,
                    container: area,
                    handle,
                });
                path.push(0);
                first.layout(a, gap, out, seams, path);
                path.pop();
                path.push(1);
                second.layout(b, gap, out, seams, path);
                path.pop();
            }
        }
    }

    /// The natural size of the subtree (plan §1.4 `Measure`): each pane
    /// at `pane(id)`, each split at the extent that gives both sides
    /// theirs ([`Split::extent_for`]) at its starting ratio (a seam the
    /// user drags does not change the size the tree asks for) and the
    /// larger side across it.
    pub fn natural_size(&self, gap: u16, pane: &dyn Fn(PaneId) -> Size) -> Size {
        match self {
            PaneNode::Leaf(id) => pane(*id),
            PaneNode::Split {
                dir,
                split,
                first,
                second,
            } => {
                let (a, b) = (
                    first.natural_size(gap, pane),
                    second.natural_size(gap, pane),
                );
                let mut split = *split;
                split.reset();
                match dir {
                    SplitDir::Horizontal => Size::new(
                        split.extent_for(a.width, b.width, gap),
                        a.height.max(b.height),
                    ),
                    SplitDir::Vertical => Size::new(
                        a.width.max(b.width),
                        split.extent_for(a.height, b.height, gap),
                    ),
                }
            }
        }
    }

    /// The node at `path` (0 = first, 1 = second at each split).
    pub fn node_at_mut(&mut self, path: &[u8]) -> Option<&mut PaneNode> {
        let mut cur = self;
        for p in path {
            match cur {
                PaneNode::Split { first, second, .. } => {
                    cur = if *p == 0 { first } else { second };
                }
                PaneNode::Leaf(_) => return None,
            }
        }
        Some(cur)
    }

    /// Replace the pane `target` with a split holding it and `new`
    /// (`new` first when `new_first`). Returns whether `target` was found.
    pub fn split_leaf(
        &mut self,
        target: PaneId,
        new: PaneId,
        dir: SplitDir,
        new_first: bool,
    ) -> bool {
        match self {
            PaneNode::Leaf(id) if *id == target => {
                let old = PaneNode::Leaf(*id);
                let fresh = PaneNode::Leaf(new);
                let (first, second) = if new_first {
                    (fresh, old)
                } else {
                    (old, fresh)
                };
                *self = PaneNode::Split {
                    dir,
                    split: Split::new(50, 1, 1),
                    first: Box::new(first),
                    second: Box::new(second),
                };
                true
            }
            PaneNode::Leaf(_) => false,
            PaneNode::Split { first, second, .. } => {
                first.split_leaf(target, new, dir, new_first)
                    || second.split_leaf(target, new, dir, new_first)
            }
        }
    }

    /// Remove a pane; its parent split collapses into the sibling.
    pub fn remove_leaf(&mut self, target: PaneId) -> bool {
        if let PaneNode::Split { first, second, .. } = self {
            if matches!(**first, PaneNode::Leaf(id) if id == target) {
                let s = std::mem::replace(&mut **second, PaneNode::Leaf(0));
                *self = s;
                return true;
            }
            if matches!(**second, PaneNode::Leaf(id) if id == target) {
                let f = std::mem::replace(&mut **first, PaneNode::Leaf(0));
                *self = f;
                return true;
            }
            return first.remove_leaf(target) || second.remove_leaf(target);
        }
        false
    }

    /// The path (0/1 steps) from the root to a pane.
    pub fn path_to(&self, target: PaneId, path: &mut Vec<u8>) -> bool {
        match self {
            PaneNode::Leaf(id) => *id == target,
            PaneNode::Split { first, second, .. } => {
                path.push(0);
                if first.path_to(target, path) {
                    return true;
                }
                path.pop();
                path.push(1);
                if second.path_to(target, path) {
                    return true;
                }
                path.pop();
                false
            }
        }
    }

    /// Every split's zoom back to none.
    fn unzoom(&mut self) {
        if let PaneNode::Split {
            split,
            first,
            second,
            ..
        } = self
        {
            split.maximized = Maximized::None;
            first.unzoom();
            second.unzoom();
        }
    }

    /// The pane every split above it maximises, if any.
    fn zoomed(&self) -> Option<PaneId> {
        match self {
            PaneNode::Leaf(id) => Some(*id),
            PaneNode::Split {
                split,
                first,
                second,
                ..
            } => match split.maximized {
                Maximized::First => first.zoomed(),
                Maximized::Second => second.zoomed(),
                Maximized::None => None,
            },
        }
    }
}

/// A seam of the tree as last laid out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Seam {
    /// The split's path in the tree.
    pub path: Vec<u8>,
    pub dir: SplitDir,
    /// The rect the split was laid out in.
    pub container: Rect,
    /// The gap strip between its panes.
    pub handle: Rect,
}

/// A direction to move focus or a seam in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
}

/// The pane nearest `from` in `dir`, by centre distance, among the laid
/// out `leaves`.
pub fn nearest(leaves: &[(PaneId, Rect)], from: PaneId, dir: Direction) -> Option<PaneId> {
    let (_, fr) = leaves.iter().find(|(id, _)| *id == from)?;
    let (fx, fy) = (
        fr.x as i32 + fr.width as i32 / 2,
        fr.y as i32 + fr.height as i32 / 2,
    );
    leaves
        .iter()
        .filter(|(id, r)| {
            *id != from
                && match dir {
                    Direction::Left => r.right() as i32 <= fr.x as i32,
                    Direction::Right => r.x as i32 >= fr.right() as i32,
                    Direction::Up => r.bottom() as i32 <= fr.y as i32,
                    Direction::Down => r.y as i32 >= fr.bottom() as i32,
                }
        })
        .min_by_key(|(_, r)| {
            let cx = r.x as i32 + r.width as i32 / 2;
            let cy = r.y as i32 + r.height as i32 / 2;
            (cx - fx).abs() + (cy - fy).abs()
        })
        .map(|(id, _)| *id)
}

/// Why a pane command did nothing; [`Self::message`] is the status line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneRefusal {
    /// Focus cannot move while a pane is zoomed.
    ZoomedFocus,
    /// Seams cannot move while a pane is zoomed.
    ZoomedResize,
    /// No pane lies that way.
    NoNeighbour,
    /// No split above the pane runs that way.
    NothingToResize,
    /// The seam would make a pane smaller than its minimum.
    AtMinimum,
    /// One pane: nothing to zoom or close.
    OnePane,
}

impl PaneRefusal {
    /// What the owner tells the user.
    pub fn message(self) -> &'static str {
        match self {
            PaneRefusal::ZoomedFocus => "Unzoom to move focus",
            PaneRefusal::ZoomedResize => "Unzoom to resize",
            PaneRefusal::NoNeighbour => "No pane that way",
            PaneRefusal::NothingToResize => "Nothing to resize in that direction",
            PaneRefusal::AtMinimum => "Cannot resize: pane at minimum size",
            PaneRefusal::OnePane => "Nothing to zoom: one pane",
        }
    }
}

/// A [`PaneTree`] command a key can be bound to (the resize chords are
/// the tree's [`SplitKeys`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneCmd {
    /// Focus the next pane (wraps).
    Next,
    /// Focus the previous pane (wraps).
    Prev,
    FocusLeft,
    FocusRight,
    FocusUp,
    FocusDown,
    /// Zoom the focused pane, or restore the layout.
    Zoom,
}

impl Actions for PaneCmd {
    /// F6 and Shift+F6 cycle the panes (the Windows and GNOME pane key);
    /// Alt+arrows move to the pane that way; Alt+Z zooms, and so does
    /// Ctrl+Shift+F12 (JetBrains' maximize-editor chord). Terminals that
    /// send Option as a character (macOS Terminal without "Use Option as
    /// Meta") do not deliver the Alt chords (A29), so hints name F6 and
    /// Ctrl+Shift+F12 there; F-keys always arrive.
    fn default_keymap() -> KeyMap<Self> {
        vec![
            (KeyBinding::key(KeyCode::F(6)), PaneCmd::Next),
            (
                KeyBinding::new(KeyCode::F(6), KeyModifiers::SHIFT),
                PaneCmd::Prev,
            ),
            (
                KeyBinding::new(KeyCode::Left, KeyModifiers::ALT),
                PaneCmd::FocusLeft,
            ),
            (
                KeyBinding::new(KeyCode::Right, KeyModifiers::ALT),
                PaneCmd::FocusRight,
            ),
            (
                KeyBinding::new(KeyCode::Up, KeyModifiers::ALT),
                PaneCmd::FocusUp,
            ),
            (
                KeyBinding::new(KeyCode::Down, KeyModifiers::ALT),
                PaneCmd::FocusDown,
            ),
            (KeyBinding::alt('z'), PaneCmd::Zoom),
            (
                KeyBinding::new(
                    KeyCode::F(12),
                    KeyModifiers::CONTROL.union(KeyModifiers::SHIFT),
                ),
                PaneCmd::Zoom,
            ),
        ]
    }

    fn label(self) -> &'static str {
        match self {
            PaneCmd::Next => "Next pane",
            PaneCmd::Prev => "Previous pane",
            PaneCmd::FocusLeft => "Pane to the left",
            PaneCmd::FocusRight => "Pane to the right",
            PaneCmd::FocusUp => "Pane above",
            PaneCmd::FocusDown => "Pane below",
            PaneCmd::Zoom => "Zoom pane",
        }
    }
}

/// When the panes get a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Frames {
    /// Once there is more than one pane, or one is zoomed (a lone pane
    /// uses every cell, as a terminal does). The default.
    #[default]
    WhenSplit,
    /// Always.
    Always,
    /// Never: the seams alone separate the panes.
    Never,
}

/// What a pane command did.
pub type PaneResult = Result<Outcome, PaneRefusal>;

/// A split tree of panes with focus, zoom, seams and frames. See the
/// module docs.
#[derive(Debug, Clone)]
pub struct PaneTree {
    /// The tree's focus id; pane `p` is the clickable part `id.child(p)`,
    /// seam `i` is `id.sub("seam").child(i)`.
    pub id: WidgetId,
    pub root: PaneNode,
    /// The focused pane.
    pub focused: PaneId,
    /// Focus and zoom keys (default [`PaneCmd::default_keymap`]).
    pub keys: KeyMap<PaneCmd>,
    /// The resize chords on the focused pane (default Alt+Shift+arrows).
    pub split_keys: SplitKeys,
    /// The smallest pane, frame included: (columns, rows). Default (12, 3).
    pub min_size: (u16, u16),
    /// When the panes are framed. Default [`Frames::WhenSplit`].
    pub frames: Frames,
    /// Seam hit padding ([`Splitter::hit_pad`]). Default 1.
    pub hit_pad: u16,
    rects: Vec<(PaneId, Rect)>,
    seams: Vec<Seam>,
    handles: Vec<Splitter>,
    /// The seam a drag holds.
    held: Option<usize>,
}

impl PaneTree {
    /// One pane, `first`, focused.
    pub fn new(id: WidgetId, first: PaneId) -> Self {
        Self {
            id,
            root: PaneNode::Leaf(first),
            focused: first,
            keys: PaneCmd::default_keymap(),
            split_keys: SplitKeys::default(),
            min_size: (12, 3),
            frames: Frames::default(),
            hit_pad: 1,
            rects: Vec::new(),
            seams: Vec::new(),
            handles: Vec::new(),
            held: None,
        }
    }

    /// See [`Self::keys`].
    pub fn keys(mut self, keys: KeyMap<PaneCmd>) -> Self {
        self.keys = keys;
        self
    }

    /// See [`Self::split_keys`].
    pub fn split_keys(mut self, keys: SplitKeys) -> Self {
        self.split_keys = keys;
        self
    }

    /// See [`Self::min_size`].
    pub fn min_size(mut self, cols: u16, rows: u16) -> Self {
        self.min_size = (cols, rows);
        self
    }

    /// See [`Self::frames`].
    pub fn frames(mut self, frames: Frames) -> Self {
        self.frames = frames;
        self
    }

    /// See [`Self::hit_pad`].
    pub fn hit_pad(mut self, cells: u16) -> Self {
        self.hit_pad = cells;
        self
    }

    /// The panes, first to last.
    pub fn leaves(&self) -> Vec<PaneId> {
        self.root.leaves()
    }

    /// Each pane's outer rect as last laid out (only the zoomed pane while
    /// one is zoomed).
    pub fn panes(&self) -> &[(PaneId, Rect)] {
        &self.rects
    }

    /// The seams as last laid out (none while a pane is zoomed).
    pub fn seams(&self) -> &[Seam] {
        &self.seams
    }

    /// The zoomed pane, if any.
    pub fn zoomed(&self) -> Option<PaneId> {
        match self.root {
            PaneNode::Leaf(_) => None,
            _ => self.root.zoomed(),
        }
    }

    /// Whether the panes are framed now.
    pub fn framed(&self) -> bool {
        match self.frames {
            Frames::Always => true,
            Frames::Never => false,
            Frames::WhenSplit => self.leaves().len() > 1 || self.zoomed().is_some(),
        }
    }

    /// Split `target` with a new pane `new` beside it (`Horizontal`) or
    /// under it (`Vertical`), `new` first when `new_first`; the new pane
    /// takes focus and any zoom ends.
    pub fn split(&mut self, target: PaneId, new: PaneId, dir: SplitDir, new_first: bool) -> bool {
        self.root.unzoom();
        let done = self.root.split_leaf(target, new, dir, new_first);
        if done {
            self.focused = new;
        }
        done
    }

    /// Close pane `target`; its sibling takes its room. The last pane does
    /// not close.
    pub fn close(&mut self, target: PaneId) -> Result<(), PaneRefusal> {
        if self.leaves().len() <= 1 {
            return Err(PaneRefusal::OnePane);
        }
        if self.zoomed() == Some(target) {
            self.root.unzoom();
        }
        if !self.root.remove_leaf(target) {
            return Err(PaneRefusal::NoNeighbour);
        }
        if self.focused == target {
            self.focused = self.leaves()[0];
        }
        Ok(())
    }

    /// Zoom the focused pane to the whole tree (every split above it
    /// maximises its side, [`Split::toggle_max`]), or restore the layout.
    pub fn toggle_zoom(&mut self) -> PaneResult {
        if self.zoomed().is_some() {
            self.root.unzoom();
            return Ok(Outcome::Changed);
        }
        if self.leaves().len() <= 1 {
            return Err(PaneRefusal::OnePane);
        }
        let mut path = vec![];
        if !self.root.path_to(self.focused, &mut path) {
            return Err(PaneRefusal::NoNeighbour);
        }
        for k in 0..path.len() {
            if let Some(PaneNode::Split { split, .. }) = self.root.node_at_mut(&path[..k]) {
                split.toggle_max(if path[k] == 0 {
                    Maximized::First
                } else {
                    Maximized::Second
                });
            }
        }
        Ok(Outcome::Changed)
    }

    /// Focus the pane `dir` of the focused one.
    pub fn focus_toward(&mut self, dir: Direction) -> PaneResult {
        if self.zoomed().is_some() {
            return Err(PaneRefusal::ZoomedFocus);
        }
        let to = nearest(&self.rects, self.focused, dir).ok_or(PaneRefusal::NoNeighbour)?;
        self.focused = to;
        Ok(Outcome::Changed)
    }

    /// Focus the next pane (or the previous one), wrapping.
    pub fn focus_next(&mut self, back: bool) -> PaneResult {
        if self.zoomed().is_some() {
            return Err(PaneRefusal::ZoomedFocus);
        }
        let leaves = self.leaves();
        if leaves.len() <= 1 {
            return Err(PaneRefusal::NoNeighbour);
        }
        let i = leaves.iter().position(|p| *p == self.focused).unwrap_or(0);
        let n = leaves.len();
        self.focused = leaves[if back { (i + n - 1) % n } else { (i + 1) % n }];
        Ok(Outcome::Changed)
    }

    /// Move the seam of the deepest split above the focused pane that
    /// runs across `dir` `cells` toward `dir` (the edge moves that way, as
    /// tmux's resize-pane does). Refused when it would leave a pane under
    /// [`Self::min_size`].
    pub fn resize(&mut self, dir: Direction, cells: u16) -> PaneResult {
        if self.zoomed().is_some() {
            return Err(PaneRefusal::ZoomedResize);
        }
        let mut path = vec![];
        if !self.root.path_to(self.focused, &mut path) {
            return Err(PaneRefusal::NothingToResize);
        }
        let want = match dir {
            Direction::Left | Direction::Right => SplitDir::Horizontal,
            Direction::Up | Direction::Down => SplitDir::Vertical,
        };
        let grows_first = matches!(dir, Direction::Right | Direction::Down);
        let mut chosen = None;
        for k in (0..path.len()).rev() {
            if let Some(PaneNode::Split { dir: sd, .. }) = self.root.node_at_mut(&path[..k])
                && *sd == want
            {
                // the seam moves toward `dir`, whichever side the pane is on
                let delta = cells.min(i16::MAX as u16) as i16;
                chosen = Some((path[..k].to_vec(), if grows_first { delta } else { -delta }));
                break;
            }
        }
        let Some((p, delta)) = chosen else {
            return Err(PaneRefusal::NothingToResize);
        };
        let Some(seam) = self.seams.iter().find(|s| s.path == p).cloned() else {
            return Err(PaneRefusal::NothingToResize);
        };
        let (min_c, min_r) = self.min_size;
        let Some(PaneNode::Split { split, dir: sd, .. }) = self.root.node_at_mut(&p) else {
            return Err(PaneRefusal::NothingToResize);
        };
        let before = *split;
        let was = split.layout(*sd, seam.container, 1);
        split.nudge(*sd, seam.container, 1, delta);
        let (a, b) = split.layout(*sd, seam.container, 1);
        let too_small = match sd {
            SplitDir::Horizontal => a.width < min_c || b.width < min_c,
            SplitDir::Vertical => a.height < min_r || b.height < min_r,
        };
        if too_small || (a, b) == was {
            *split = before;
            return Err(PaneRefusal::AtMinimum);
        }
        Ok(Outcome::Changed)
    }

    /// The natural size of the tree (plan §1.4 `Measure`): the size whose
    /// [`Self::render`] gives each pane a content rect of at least
    /// `content(pane)`, its frame included; only the zoomed pane while
    /// one is zoomed.
    pub fn natural_size(&self, content: &dyn Fn(PaneId) -> Size) -> Size {
        let frame = if self.framed() { 2 } else { 0 };
        let pane = |id: PaneId| {
            let c = content(id);
            Size::new(c.width + frame, c.height + frame)
        };
        match self.zoomed() {
            Some(z) => pane(z),
            None => self.root.natural_size(1, &pane),
        }
    }

    /// Lay the panes out in `area`: [`Self::panes`] and [`Self::seams`].
    pub fn layout(&mut self, area: Rect) {
        self.rects.clear();
        self.seams.clear();
        let mut rects = Vec::new();
        let mut seams = Vec::new();
        self.root
            .layout(area, 1, &mut rects, &mut seams, &mut Vec::new());
        // a zoomed tree lays out only the zoomed pane and no seams
        let zoomed = self.zoomed();
        self.rects = rects
            .into_iter()
            .filter(|(id, r)| zoomed.is_none_or(|z| z == *id) && !r.is_empty())
            .collect();
        if zoomed.is_none() {
            self.seams = seams.into_iter().filter(|s| !s.handle.is_empty()).collect();
        }
    }

    /// The seam handle of seam `i`.
    pub fn seam_id(&self, i: usize) -> WidgetId {
        self.id.sub("seam").child(i)
    }

    /// Lay out and draw the tree in `area` on `bg`: the seams, and each
    /// pane's frame with the title `title(pane)` (the focused pane's frame
    /// strong while the tree holds focus, a hovered pane's title
    /// brighter). Registers the tree as a focus stop and each pane as a
    /// clickable part. Returns each pane's content rect for the owner to
    /// draw into.
    pub fn render(
        &mut self,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
        title: &dyn Fn(PaneId) -> String,
    ) -> Vec<(PaneId, Rect)> {
        self.layout(area);
        let t = ctx.theme;
        ctx.control(self.id, area, false);
        let tree_focused = ctx.interaction.focused(self.id);
        let framed = self.framed();
        // the seams: one Splitter each, keeping a held drag across frames
        let n = self.seams.len();
        self.handles.truncate(n);
        for i in self.handles.len()..n {
            let dir = self.seams[i].dir;
            self.handles
                .push(Splitter::new(self.seam_id(i), dir).hit_pad(self.hit_pad));
        }
        for i in 0..n {
            let s = &self.seams[i];
            let mut h = Splitter::new(self.seam_id(i), s.dir).hit_pad(self.hit_pad);
            if self.handles[i].dir == s.dir {
                h = self.handles[i];
                h.hit_pad = self.hit_pad;
            }
            h.render_held(s.handle, buf, ctx, bg, self.held == Some(i));
            self.handles[i] = h;
        }
        let mut inner = Vec::with_capacity(self.rects.len());
        let zoomed = self.zoomed().is_some();
        for &(pid, r) in &self.rects {
            let part = self.id.child(pid as usize);
            ctx.clickable(part, r);
            if !framed {
                inner.push((pid, r));
                continue;
            }
            let focused = tree_focused && pid == self.focused;
            let hovered = ctx.interaction.hovered(part);
            // the frame is a framed Panel: the theme's border glyphs, the
            // strong border on the focused pane
            let name = title(pid);
            let meta = if zoomed { Some("zoomed") } else { None };
            let mut panel = crate::widgets::panel::Panel::framed(Some(&name)).focused(focused);
            panel.bg_override = Some(bg);
            if let Some(m) = meta {
                panel = panel.meta(m);
            }
            panel.render(r, buf, t);
            if hovered && !focused && r.width > 4 {
                // a hovered pane's title brightens (the pointer target)
                let w = crate::ui::text::width(&name) as u16 + 2;
                for x in r.x + 2..(r.x + 2 + w).min(r.right().saturating_sub(2)) {
                    buf[(x, r.y)].set_style(t.primary().bg(bg));
                }
            }
            inner.push((pid, r.inner(Margin::new(1, 1))));
        }
        inner
    }

    /// The tree's own keys while it holds focus: the pane keys
    /// ([`Self::keys`]) and the resize chords ([`Self::split_keys`]) on
    /// the focused pane. `None` for any other key: the focused pane's
    /// content has it.
    pub fn on_key(&mut self, key: &Key) -> Option<PaneResult> {
        if let Some(cmd) = lookup(&self.keys, key) {
            return Some(match cmd {
                PaneCmd::Next => self.focus_next(false),
                PaneCmd::Prev => self.focus_next(true),
                PaneCmd::FocusLeft => self.focus_toward(Direction::Left),
                PaneCmd::FocusRight => self.focus_toward(Direction::Right),
                PaneCmd::FocusUp => self.focus_toward(Direction::Up),
                PaneCmd::FocusDown => self.focus_toward(Direction::Down),
                PaneCmd::Zoom => self.toggle_zoom(),
            });
        }
        let dir = [
            (SplitDir::Horizontal, Direction::Left, Direction::Right),
            (SplitDir::Vertical, Direction::Up, Direction::Down),
        ]
        .into_iter()
        .find_map(|(axis, back, fwd)| {
            self.split_keys
                .delta(key, axis)
                .map(|d| if d < 0 { back } else { fwd })
        })?;
        Some(self.resize(dir, self.split_keys.step))
    }

    /// A press, drag or click on a seam (drags cell by cell, a
    /// double-click restores its starting ratio), or a click on a pane
    /// (focuses it). `None` when the event is not the tree's.
    pub fn on_pointer(&mut self, ev: &PointerEvent) -> Option<Outcome> {
        if let Some(i) = (0..self.seams.len()).find(|&i| self.seam_id(i) == ev.id) {
            let s = self.seams[i].clone();
            let mut h = self.handles.get(i).copied()?;
            let Some(PaneNode::Split { split, .. }) = self.root.node_at_mut(&s.path) else {
                return None;
            };
            let o = match ev.phase()? {
                Phase::Press => {
                    self.held = Some(i);
                    h.on_press(split, s.container, 1, ev.pos());
                    Outcome::Consumed
                }
                Phase::Drag => {
                    self.held = Some(i);
                    h.on_drag(split, s.container, 1, ev.pos())
                }
                Phase::Click => {
                    self.held = None;
                    h.on_click(split, ev.count())
                }
                _ => return None,
            };
            self.handles[i] = h;
            return Some(o);
        }
        if ev.is(Phase::Click)
            && let Some(&(pid, _)) = self
                .rects
                .iter()
                .find(|(p, _)| self.id.child(*p as usize) == ev.id)
        {
            self.focused = pid;
            return Some(Outcome::Changed);
        }
        None
    }

    /// The footer hints for the tree's keys while it holds focus.
    pub fn hints(&self) -> Vec<Hint> {
        let mut v = Vec::new();
        if self.zoomed().is_none() {
            if let Some(k) = hint_key(&self.keys, PaneCmd::Next) {
                v.push(hint(k, "Next pane"));
            }
            if let Some(h) = self.split_keys.hint(SplitDir::Horizontal) {
                v.push(h);
            }
        }
        if self.leaves().len() > 1
            && let Some(k) = hint_key(&self.keys, PaneCmd::Zoom)
        {
            v.push(hint(
                k,
                if self.zoomed().is_some() {
                    "Restore"
                } else {
                    "Zoom"
                },
            ));
        }
        v
    }
}

/// In a States grid: two panes side by side, the left one focused or
/// hovered, and a zoomed pane.
impl crate::ui::specimen::Specimen for PaneTree {
    fn specimen_id(&self) -> WidgetId {
        self.id
    }

    fn supports(&self, state: crate::ui::specimen::SpecimenState) -> bool {
        use crate::ui::specimen::SpecimenState::*;
        matches!(state, Rest | Hover | Focused | Selected)
    }

    fn specimen_height(&self, _state: crate::ui::specimen::SpecimenState) -> u16 {
        4
    }

    fn specimen(
        &self,
        state: crate::ui::specimen::SpecimenState,
        area: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        bg: Color,
    ) {
        use crate::ui::specimen::SpecimenState;
        let mut tree = PaneTree::new(self.id, 1).min_size(4, 3);
        tree.split(1, 2, SplitDir::Horizontal, false);
        tree.focused = 1;
        if state == SpecimenState::Selected {
            let _ = tree.toggle_zoom();
        }
        let saved = ctx.interaction;
        match state {
            SpecimenState::Focused => ctx.interaction.focus = Some(self.id),
            SpecimenState::Hover => ctx.interaction.hover = Some(self.id.child(1)),
            _ => {}
        }
        let inert = std::mem::replace(&mut ctx.inert, true);
        let names = |p: PaneId| {
            if p == 1 {
                "shell".to_owned()
            } else {
                "logs".to_owned()
            }
        };
        tree.render(area, buf, ctx, bg, &names);
        ctx.inert = inert;
        ctx.interaction = saved;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::focus::FocusRing;
    use crate::core::hit::HitRegistry;
    use crate::theme::{ColorLevel, Theme};
    use crate::ui::ctx::Interaction;
    use ratatui::layout::Position;

    const ID: WidgetId = WidgetId::of("panes");

    /// shell | (build over logs)
    fn three() -> PaneTree {
        let mut t = PaneTree::new(ID, 1);
        t.split(1, 2, SplitDir::Horizontal, false);
        t.split(2, 3, SplitDir::Vertical, false);
        t.layout(Rect::new(0, 0, 80, 24));
        t
    }

    fn key(code: KeyCode, mods: KeyModifiers) -> Key {
        Key { code, mods }
    }

    fn rect(t: &PaneTree, p: PaneId) -> Rect {
        t.panes().iter().find(|(id, _)| *id == p).unwrap().1
    }

    /// Laid out at its natural size, every pane's content rect is at
    /// least what it asked for; zoomed, the zoomed pane's alone.
    #[test]
    fn a_tree_at_its_natural_size_gives_every_pane_its_content() {
        let mut t = three();
        let want = |p: PaneId| Size::new(10 + p as u16 * 7, 3 + p as u16);
        let size = t.natural_size(&want);
        t.layout(Rect::new(0, 0, size.width, size.height));
        let frame = u16::from(t.framed()) * 2;
        for (p, r) in t.panes().to_vec() {
            let w = want(p);
            assert!(
                r.width >= w.width + frame && r.height >= w.height + frame,
                "pane {p}: {r:?} for {w:?} in {size:?}"
            );
        }
        t.toggle_zoom().unwrap();
        let z = t.zoomed().unwrap();
        assert_eq!(
            t.natural_size(&want),
            Size::new(want(z).width + frame, want(z).height + frame)
        );
    }

    #[test]
    fn nested_splits_lay_out_every_pane_and_seam() {
        let t = three();
        assert_eq!(t.leaves(), vec![1, 2, 3]);
        assert_eq!(t.panes().len(), 3);
        assert_eq!(t.seams().len(), 2);
        assert_eq!(rect(&t, 2).x, rect(&t, 3).x, "build is over logs");
        assert!(rect(&t, 2).bottom() < rect(&t, 3).y);
        assert_eq!(t.focused, 3, "the new pane takes focus");
    }

    #[test]
    fn focus_moves_by_direction_and_cycles() {
        let mut t = three();
        assert_eq!(t.focus_toward(Direction::Left), Ok(Outcome::Changed));
        assert_eq!(t.focused, 1);
        assert_eq!(
            t.focus_toward(Direction::Left),
            Err(PaneRefusal::NoNeighbour)
        );
        assert_eq!(
            t.on_key(&key(KeyCode::F(6), KeyModifiers::NONE)),
            Some(Ok(Outcome::Changed))
        );
        assert_eq!(t.focused, 2);
        t.on_key(&key(KeyCode::F(6), KeyModifiers::SHIFT));
        assert_eq!(t.focused, 1);
        t.on_key(&key(KeyCode::Right, KeyModifiers::ALT));
        assert!(matches!(t.focused, 2 | 3), "a pane on the right");
        t.on_key(&key(KeyCode::Up, KeyModifiers::ALT));
        assert_eq!(t.focused, 2);
        t.on_key(&key(KeyCode::Down, KeyModifiers::ALT));
        assert_eq!(t.focused, 3);
    }

    #[test]
    fn zoom_maximises_the_focused_pane_through_its_splits() {
        let mut t = three();
        t.focused = 2;
        assert_eq!(t.toggle_zoom(), Ok(Outcome::Changed));
        assert_eq!(t.zoomed(), Some(2));
        t.layout(Rect::new(0, 0, 80, 24));
        assert_eq!(t.panes(), &[(2, Rect::new(0, 0, 80, 24))]);
        assert!(t.seams().is_empty());
        assert_eq!(
            t.focus_toward(Direction::Left),
            Err(PaneRefusal::ZoomedFocus)
        );
        assert_eq!(t.resize(Direction::Left, 2), Err(PaneRefusal::ZoomedResize));
        // Alt+Z restores
        assert_eq!(
            t.on_key(&key(KeyCode::Char('z'), KeyModifiers::ALT)),
            Some(Ok(Outcome::Changed))
        );
        assert_eq!(t.zoomed(), None);
        let mut one = PaneTree::new(ID, 1);
        assert_eq!(one.toggle_zoom(), Err(PaneRefusal::OnePane));
    }

    #[test]
    fn resize_moves_the_nearest_seam_across_the_axis() {
        let mut t = three();
        t.focused = 1;
        let w = rect(&t, 1).width;
        let chord = key(KeyCode::Right, KeyModifiers::ALT | KeyModifiers::SHIFT);
        assert_eq!(t.on_key(&chord), Some(Ok(Outcome::Changed)));
        t.layout(Rect::new(0, 0, 80, 24));
        assert_eq!(rect(&t, 1).width, w + 2);
        // the stacked panes resize on the vertical seam
        t.focused = 2;
        let h = rect(&t, 2).height;
        t.on_key(&key(KeyCode::Down, KeyModifiers::ALT | KeyModifiers::SHIFT));
        t.layout(Rect::new(0, 0, 80, 24));
        assert_eq!(rect(&t, 2).height, h + 2);
        // the left pane has no vertical seam above it
        t.focused = 1;
        assert_eq!(
            t.resize(Direction::Up, 2),
            Err(PaneRefusal::NothingToResize)
        );
        // a minimum refuses
        let mut tight = three().min_size(39, 3);
        tight.focused = 1;
        assert_eq!(
            tight.resize(Direction::Left, 4),
            Err(PaneRefusal::AtMinimum)
        );
    }

    /// A29: on macOS Terminal (Option types a character) the hints name
    /// F6, Ctrl+Shift+arrows and Ctrl+Shift+F12, and Ctrl+Shift+F12
    /// zooms; with the kitty protocol the Alt chords are named.
    #[test]
    fn hints_name_no_alt_chord_where_the_terminal_keeps_option() {
        use crate::core::keyboard::{self, AltKeys, KeyboardProtocol};
        let keys_of = |t: &PaneTree| t.hints().iter().map(|h| h.key).collect::<Vec<_>>();
        let mut t = three();
        keyboard::set_alt_keys(AltKeys::from_env(
            crate::core::event::ClientPlatform::MacOs,
            |k| (k == "TERM_PROGRAM").then(|| "Apple_Terminal".to_owned()),
        ));
        assert_eq!(keys_of(&t), ["F6", "Ctrl+Shift+←→", "Ctrl+Shift+F12"]);
        let zoom = key(KeyCode::F(12), KeyModifiers::CONTROL | KeyModifiers::SHIFT);
        assert_eq!(t.on_key(&zoom), Some(Ok(Outcome::Changed)));
        assert!(t.zoomed().is_some());
        t.on_key(&zoom);
        keyboard::set_alt_keys(AltKeys::WithProtocol);
        keyboard::session::enter(KeyboardProtocol::On, || false);
        assert_eq!(keys_of(&t), ["F6", "Alt+Shift+←→", "Alt+Z"]);
        keyboard::session::leave();
    }

    #[test]
    fn close_collapses_into_the_sibling_and_keeps_the_last_pane() {
        let mut t = three();
        assert_eq!(t.close(3), Ok(()));
        assert_eq!(t.leaves(), vec![1, 2]);
        assert_eq!(t.focused, 1);
        assert_eq!(t.close(2), Ok(()));
        assert_eq!(t.close(1), Err(PaneRefusal::OnePane));
    }

    #[test]
    fn render_frames_the_panes_and_a_click_focuses_one() {
        let th = Theme::for_level(ColorLevel::TrueColor);
        let mut t = three();
        let mut hits = HitRegistry::default();
        let mut ring = FocusRing::default();
        let interaction = Interaction {
            focus: Some(ID),
            ..Default::default()
        };
        let mut ctx = RenderCtx::new(&th, interaction, &mut hits, &mut ring);
        let area = Rect::new(0, 0, 80, 24);
        let mut buf = Buffer::empty(area);
        let names = |p: PaneId| format!("pane {p}");
        let inner = t.render(area, &mut buf, &mut ctx, th.canvas, &names);
        drop(ctx);
        assert_eq!(inner.len(), 3);
        assert_eq!(buf[(0, 0)].symbol(), "╭");
        let row: String = (0..80).map(|x| buf[(x, 0)].symbol().to_owned()).collect();
        assert!(row.contains("pane 1"), "{row}");
        // the focused pane's frame is the strong border
        let (_, r3) = t.panes()[2];
        assert_eq!(buf[(r3.x, r3.y)].fg, th.border_strong);
        let (_, r1) = t.panes()[0];
        assert_eq!(buf[(r1.x, r1.y)].fg, th.border_subtle);
        let click = PointerEvent::new(
            &hits,
            Phase::Click,
            ID.child(1),
            Position::new(r1.x + 3, r1.y + 3),
            1,
            KeyModifiers::NONE,
        );
        assert_eq!(t.on_pointer(&click), Some(Outcome::Changed));
        assert_eq!(t.focused, 1);
        // a seam drags
        let s = t.seams()[0].handle;
        let press = PointerEvent::new(
            &hits,
            Phase::Press,
            t.seam_id(0),
            Position::new(s.x, 5),
            1,
            KeyModifiers::NONE,
        );
        t.on_pointer(&press);
        let drag = PointerEvent::new(
            &hits,
            Phase::Drag,
            t.seam_id(0),
            Position::new(s.x + 5, 5),
            1,
            KeyModifiers::NONE,
        );
        assert_eq!(t.on_pointer(&drag), Some(Outcome::Changed));
        let w = r1.width;
        t.layout(area);
        assert_eq!(rect(&t, 1).width, w + 5);
    }
}
