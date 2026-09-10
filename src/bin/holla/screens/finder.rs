//! The finder: the Here tab's root page and every domain page. A live query
//! row, grouped fixed-column results (`label · type · scope · reason`), a
//! preview card that answers CONCEPT §7, and the scope readout. Typing-hot:
//! every printable key edits the query.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::{Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, truncate_middle, width, wrap};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::action::{Confirmation, Item, Kind, Launch, Risk};
use crate::domain::context::Scope;
use crate::domain::ranking::{Ranked, search};
use crate::screens::{Cx, Go, Page, Screen, StatusBits, heading, plural, risk_tone, truncate_sep};
use crate::sim::world::World;

pub const FINDER: WidgetId = WidgetId::of("here.finder");
pub const PREVIEW: WidgetId = WidgetId::of("here.preview");
pub const SCOPE: WidgetId = WidgetId::of("here.scope");
const EXPLORE_CELL: WidgetId = WidgetId::of("here.explore");

/// Preview beside the list from this body width (a 100-column terminal
/// minus the shell's side margins); a drawer below it.
pub const SPLIT_MIN: u16 = 108;
/// Commands shown in the preview before `… N more`.
const PREVIEW_COMMANDS: usize = 6;
/// A preview line: label, value, tone, wraps (prose) or truncates in the middle.
type PreviewLine = (String, String, Tone, bool);

#[derive(Debug, Clone, PartialEq, Eq)]
enum Row {
    Heading(String),
    Item(usize, Ranked),
    /// The Explore domains as one row of cells.
    Explore(Vec<usize>),
    Blank,
}

pub struct FinderPage {
    pub group: Option<&'static str>,
    pub query: String,
    pub scope: Scope,
    items: Vec<Item>,
    rows: Vec<Row>,
    pub cursor: usize,
    scroll: ScrollState,
    preview_scroll: ScrollState,
    explore_cursor: usize,
    /// Preview drawer open (narrow layouts only).
    pub drawer: bool,
    list_area: Rect,
    preview_area: Rect,
    scope_area: Rect,
    explore_areas: Vec<(WidgetId, Rect, usize)>,
    split: bool,
    /// Rows are rebuilt when this changes.
    dirty: bool,
    last_tick: u64,
}

impl FinderPage {
    pub fn new(group: Option<&'static str>) -> Self {
        Self {
            group,
            query: String::new(),
            scope: Scope::Here,
            items: vec![],
            rows: vec![],
            cursor: 0,
            scroll: ScrollState::default(),
            preview_scroll: ScrollState::default(),
            explore_cursor: 0,
            drawer: false,
            list_area: Rect::ZERO,
            preview_area: Rect::ZERO,
            scope_area: Rect::ZERO,
            explore_areas: vec![],
            split: true,
            dirty: true,
            last_tick: u64::MAX,
        }
    }

    /// The item under the cursor.
    pub fn current(&self) -> Option<&Item> {
        match self.rows.get(self.cursor) {
            Some(Row::Item(i, _)) => self.items.get(*i),
            Some(Row::Explore(cells)) => cells
                .get(self.explore_cursor)
                .and_then(|i| self.items.get(*i)),
            _ => None,
        }
    }

    pub fn current_id(&self) -> Option<String> {
        self.current().map(|i| i.id.clone())
    }

    pub fn set_scope(&mut self, s: Scope) {
        self.scope = s;
        self.dirty = true;
    }

    fn rebuild(&mut self, w: &World) {
        let keep = self.current_id();
        self.items = w.items();
        let ranked = search(&self.items, &self.query, self.scope, self.group);
        let has_query = !crate::domain::ranking::Query::parse(&self.query).is_empty();
        let mut rows = vec![];
        if let Some(g) = self.group {
            rows.push(Row::Heading(format!(
                "{g} · {}",
                plural(ranked.len(), "entry", "entries")
            )));
            for r in ranked {
                rows.push(Row::Item(r.index, r));
            }
        } else if has_query || self.scope != Scope::Here {
            let title = if has_query {
                format!("Results · {}", ranked.len())
            } else {
                format!(
                    "{} · {}",
                    self.scope_title(w),
                    plural(ranked.len(), "entry", "entries")
                )
            };
            rows.push(Row::Heading(title));
            for r in ranked
                .into_iter()
                .filter(|r| has_query || self.items[r.index].kind != Kind::Explore)
                .take(60)
            {
                rows.push(Row::Item(r.index, r));
            }
        } else {
            let mut used = std::collections::BTreeSet::new();
            // suggested: recommended rows with a live or learned signal
            let suggested: Vec<&Ranked> = ranked
                .iter()
                .filter(|r| {
                    let it = &self.items[r.index];
                    it.is_recommended() && !matches!(it.kind, Kind::Explore | Kind::Activity)
                })
                .take(5)
                .collect();
            if !suggested.is_empty() {
                rows.push(Row::Heading("Suggested here".into()));
                for r in suggested {
                    used.insert(r.index);
                    rows.push(Row::Item(r.index, r.clone()));
                }
            }
            // recent here: by last use, not already shown
            let mut recent: Vec<&Ranked> = ranked
                .iter()
                .filter(|r| {
                    self.items[r.index].last_used_secs.is_some() && !used.contains(&r.index)
                })
                .collect();
            recent.sort_by_key(|r| {
                std::cmp::Reverse(self.items[r.index].last_used_secs.unwrap_or(0))
            });
            let recent: Vec<&Ranked> = recent.into_iter().take(3).collect();
            rows.push(Row::Blank);
            rows.push(Row::Heading("Recent here".into()));
            if recent.is_empty() {
                rows.push(Row::Heading("nothing used here yet".into()));
            }
            for r in recent {
                used.insert(r.index);
                let mut rr = r.clone();
                if let Some(secs) = self.items[r.index].last_used_secs {
                    rr.reason = w.clock.ago(secs);
                }
                rows.push(Row::Item(r.index, rr));
            }
            // what the parent root contributes to this folder (CONCEPT §5.3)
            if w.location.workspace.is_some() {
                let parent_ranked = search(&self.items, "", Scope::Parent, None);
                // the root's own tasks (ecosystem, containers, setup) first,
                // then whatever else the parent contributes
                let eligible = |r: &&Ranked| {
                    let it = &self.items[r.index];
                    it.scope.word == "parent"
                        && it.kind != Kind::Explore
                        && !used.contains(&r.index)
                };
                let ecosystem_rank = |r: &&Ranked| {
                    let id = self.items[r.index].id.as_str();
                    match id.rsplit(['.', ':']).next().unwrap_or("") {
                        "dev" => 0,
                        "up" => 1,
                        "setup" => 2,
                        _ => 3,
                    }
                };
                let mut tasks: Vec<&Ranked> = parent_ranked
                    .iter()
                    .filter(eligible)
                    .filter(|r| self.items[r.index].kind == Kind::Mise)
                    .collect();
                tasks.sort_by_key(|r| ecosystem_rank(r));
                let mut parent: Vec<&Ranked> = tasks.into_iter().take(2).collect();
                for r in parent_ranked.iter().filter(eligible) {
                    if parent.len() >= 2 {
                        break;
                    }
                    if !parent.iter().any(|p| p.index == r.index) {
                        parent.push(r);
                    }
                }
                if !parent.is_empty() {
                    let name = w
                        .location
                        .workspace
                        .as_ref()
                        .map(|p| p.name.clone())
                        .unwrap_or_default();
                    rows.push(Row::Blank);
                    rows.push(Row::Heading(format!("From {name} · the parent root")));
                    for r in parent {
                        used.insert(r.index);
                        rows.push(Row::Item(r.index, r.clone()));
                    }
                }
            }
            // running activities
            let running: Vec<&Ranked> = ranked
                .iter()
                .filter(|r| self.items[r.index].kind == Kind::Activity)
                .take(4)
                .collect();
            if !running.is_empty() {
                rows.push(Row::Blank);
                rows.push(Row::Heading("Running".into()));
                for r in running {
                    used.insert(r.index);
                    rows.push(Row::Item(r.index, r.clone()));
                }
            }
            // explore: domain cells then scope rows
            rows.push(Row::Blank);
            rows.push(Row::Heading("Explore".into()));
            let cells: Vec<usize> = self
                .items
                .iter()
                .enumerate()
                .filter(|(_, it)| it.kind == Kind::Explore && it.id.starts_with("explore."))
                .map(|(i, _)| i)
                .collect();
            if !cells.is_empty() {
                rows.push(Row::Explore(cells));
            }
            for r in ranked
                .iter()
                .filter(|r| self.items[r.index].id.starts_with("scope."))
            {
                rows.push(Row::Item(r.index, r.clone()));
            }
        }
        self.rows = rows;
        // keep the selection stable across rebuilds (CONCEPT §5.5 #9)
        let target = keep.and_then(|id| {
            self.rows
                .iter()
                .position(|r| matches!(r, Row::Item(i, _) if self.items[*i].id == id))
        });
        self.cursor = target.unwrap_or_else(|| self.first_selectable());
        self.scroll.set_content(self.rows.len());
        self.dirty = false;
    }

    fn first_selectable(&self) -> usize {
        self.rows
            .iter()
            .position(|r| matches!(r, Row::Item(..) | Row::Explore(_)))
            .unwrap_or(0)
    }

    fn scope_title(&self, w: &World) -> String {
        match self.scope {
            Scope::Here => "Here".into(),
            Scope::Parent => match &w.location.workspace {
                Some(p) => format!("Parent · {}", p.name),
                None => match &w.location.project {
                    Some(p) => format!("Parent · {}", p.name),
                    None => "Parent".into(),
                },
            },
            Scope::Children => format!(
                "Children · {}",
                plural(w.location.children.len(), "project", "projects")
            ),
            Scope::System => format!("System · {}", w.host.name),
        }
    }

    fn step(&mut self, delta: isize) {
        if self.rows.is_empty() {
            return;
        }
        let n = self.rows.len() as isize;
        let mut c = self.cursor as isize;
        loop {
            c += delta;
            if c < 0 || c >= n {
                return;
            }
            if matches!(self.rows[c as usize], Row::Item(..) | Row::Explore(_)) {
                self.cursor = c as usize;
                self.preview_scroll.jump_start();
                self.scroll.ensure_visible(self.cursor);
                // keep the heading above the first row in view
                if self.cursor > 0
                    && self.cursor == self.scroll.offset
                    && matches!(self.rows[self.cursor - 1], Row::Heading(_))
                {
                    self.scroll.scroll_by(-1);
                }
                return;
            }
        }
    }

    fn set_query(&mut self, q: String) {
        self.query = q;
        self.dirty = true;
        self.scroll.jump_start();
    }

    fn primary_hint(&self) -> Hint {
        match self.current() {
            Some(it) => match (&it.launch, &it.confirmation) {
                (_, Confirmation::TwoGate { .. }) => hint("Enter", "Review…"),
                (_, Confirmation::Trust { .. }) => hint("Enter", "Trust…"),
                (Launch::Plan { .. }, _) => hint("Enter", "Review plan"),
                (
                    Launch::Disk { .. }
                    | Launch::Group(_)
                    | Launch::Scope(_)
                    | Launch::Snapshot { .. },
                    _,
                ) => hint("Enter", "Open"),
                (Launch::OpenActivity { .. }, _) => hint("Enter", "Switch"),
                (_, Confirmation::One) => hint("Enter", "Confirm…"),
                _ => hint("Enter", "Run"),
            },
            None => hint("Enter", "Run"),
        }
    }

    /// The preview's lines: label, value, tone, and whether the value wraps
    /// (prose) or is truncated in the middle (paths and commands).
    fn preview_lines(&self, it: &Item, w: &World) -> Vec<PreviewLine> {
        let mut v: Vec<PreviewLine> = vec![];
        v.push(("Does".into(), it.summary.clone(), Tone::Normal, true));
        for (i, c) in it.commands.iter().enumerate().take(PREVIEW_COMMANDS) {
            v.push((
                if i == 0 { "Runs".into() } else { String::new() },
                c.clone(),
                Tone::Secondary,
                false,
            ));
        }
        if it.commands.len() > PREVIEW_COMMANDS {
            v.push((
                String::new(),
                format!("… {} more", it.commands.len() - PREVIEW_COMMANDS),
                Tone::Muted,
                false,
            ));
        }
        if it.commands.is_empty() {
            v.push((
                "Runs".into(),
                "no command · opens inside holla".into(),
                Tone::Muted,
                true,
            ));
        }
        let runs_in = if it.scope.direction == Scope::System {
            format!("{} · host-wide", w.host.name)
        } else {
            w.location.short(&it.scope.runs_in)
        };
        v.push(("In".into(), runs_in, Tone::Normal, false));
        v.push((
            "Host".into(),
            format!(
                "{} · {}{}",
                w.host.name,
                w.host.role.label(),
                if w.host.remote { " · over SSH" } else { "" }
            ),
            Tone::Normal,
            true,
        ));
        // the defining path is named only when it differs from `In`
        let defined =
            if it.scope.direction == Scope::System || it.scope.defined_at == it.scope.runs_in {
                String::new()
            } else {
                format!(" · {}", w.location.short(&it.scope.defined_at))
            };
        v.push((
            "Scope".into(),
            format!("{}{}", it.scope.word, defined),
            Tone::Normal,
            false,
        ));
        let mut reasons: Vec<&str> = it.reasons.iter().map(|r| r.text.as_str()).collect();
        reasons.sort();
        let why = {
            let mut sorted: Vec<&crate::domain::action::Reason> = it.reasons.iter().collect();
            sorted.sort_by_key(|r| std::cmp::Reverse(r.signal));
            sorted
                .iter()
                .map(|r| r.text.as_str())
                .take(3)
                .collect::<Vec<_>>()
                .join(" · ")
        };
        v.push((
            "Why".into(),
            if why.is_empty() {
                "available".into()
            } else {
                why
            },
            Tone::Normal,
            true,
        ));
        let changes = if it.effects.is_empty() {
            match it.risk {
                Risk::ReadOnly => "none · read-only".to_owned(),
                _ => "see the command".to_owned(),
            }
        } else {
            it.effects.join(" · ")
        };
        v.push(("Changes".into(), changes, Tone::Normal, true));
        v.push((
            "Risk".into(),
            it.risk.label().into(),
            risk_tone(it.risk),
            true,
        ));
        v.push((
            "Data".into(),
            it.freshness.label(),
            match it.freshness {
                crate::domain::action::Freshness::Unavailable(_) => Tone::Error,
                crate::domain::action::Freshness::Loading
                | crate::domain::action::Freshness::Partial { .. } => Tone::Warning,
                _ => Tone::Muted,
            },
            true,
        ));
        let gate = match &it.confirmation {
            Confirmation::TwoGate { .. } => format!(
                "two gates · type {}",
                it.confirmation.phrase().unwrap_or_default()
            ),
            c => c.label(),
        };
        v.push((
            "Gate".into(),
            gate,
            if matches!(it.confirmation, Confirmation::None) {
                Tone::Muted
            } else {
                Tone::Normal
            },
            true,
        ));
        if !it.args.is_empty() {
            let names: Vec<String> = it
                .args
                .iter()
                .map(|a| {
                    if a.default.is_empty() {
                        a.name.clone()
                    } else {
                        format!("{} = {}", a.name, a.default)
                    }
                })
                .collect();
            v.push(("Args".into(), names.join(" · "), Tone::Normal, true));
        }
        if let Some(tool) = &it.preferred_tool {
            v.push(("Prefers".into(), tool.clone(), Tone::Muted, true));
        }
        let mut alts: Vec<String> = it.alternatives.iter().map(|a| a.label.clone()).collect();
        alts.push("Pin".into());
        alts.push("Alias".into());
        alts.push("Why here?".into());
        v.push(("Alt+Enter".into(), alts.join(" · "), Tone::Muted, true));
        let _ = reasons;
        v
    }

    fn render_preview(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(PREVIEW);
        let Some(it) = self.current().cloned() else {
            let hint_text = if self.rows.iter().any(|r| matches!(r, Row::Item(..))) {
                "↑↓ moves the cursor"
            } else if self.query.trim().is_empty() {
                "Ctrl+↑ widens the scope"
            } else {
                "Esc clears the query"
            };
            let card_h = 7.min(area.height);
            let area = Rect::new(area.x, area.y, area.width, card_h);
            let panel = Panel::card(Some("Preview")).focused(focused);
            let inner = panel.render(area, buf, t);
            empty::render(
                inner,
                buf,
                t,
                &EmptyState::new("Nothing to preview").hint(hint_text),
                panel.bg(t),
            );
            ctx.control(PREVIEW, area, false);
            return;
        };
        let meta = format!("{} · {}", it.kind.label(), it.scope.word);
        let panel = Panel::card(Some(&it.label)).focused(focused).meta(&meta);
        let bg = panel.bg(t);
        // lines first: the card ends after its content
        let lines = self.preview_lines(&it, w);
        let label_w = lines.iter().map(|(l, ..)| width(l)).max().unwrap_or(4) as u16 + 2;
        let inner_w = area.width.saturating_sub(4);
        let vw = inner_w.saturating_sub(label_w + 1) as usize;
        let mut flat: Vec<(String, String, Tone)> = vec![];
        for (l, v, tone, wraps) in lines {
            if wraps {
                let parts = wrap(&v, vw.max(8));
                for (i, p) in parts.into_iter().enumerate() {
                    flat.push((if i == 0 { l.clone() } else { String::new() }, p, tone));
                }
            } else {
                flat.push((l, truncate_middle(&v, vw.max(8)), tone));
            }
        }
        let card_h = (flat.len() as u16 + 3).min(area.height);
        let area = Rect::new(area.x, area.y, area.width, card_h);
        let inner = panel.render(area, buf, t);
        self.preview_area = area;
        ctx.control(PREVIEW, area, false);
        ctx.scrollable(PREVIEW, inner);
        if inner.is_empty() {
            return;
        }
        self.preview_scroll.set_content(flat.len());
        self.preview_scroll.set_viewport(inner.height as usize);
        let has_sb = self.preview_scroll.overflows();
        for (k, i) in self.preview_scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let (l, v, tone) = &flat[i];
            buf.set_string(inner.x, y, l, t.muted().bg(bg));
            let st = Style::new().fg(t.tone(*tone)).bg(bg);
            buf.set_string(inner.x + label_w, y, truncate(v, vw), st);
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                PREVIEW,
                &self.preview_scroll,
                focused,
            );
        }
    }

    fn render_query(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(FINDER);
        let fs = t.field_style(junie_tui::ui::ctx::VisualState {
            focused,
            editing: focused,
            ..Default::default()
        });
        fill(buf, area, fs);
        let bg = fs.bg.unwrap_or(t.field);
        buf.set_string(
            area.x,
            area.y,
            if focused { "▎" } else { " " },
            if focused {
                Style::new().fg(t.focus).bg(bg)
            } else {
                Style::new().fg(bg).bg(bg)
            },
        );
        // scope readout on the right: ` scope ‹ here › `
        let readout = format!("scope ‹ {} ›", self.scope_word());
        let rw = width(&readout) as u16 + 2;
        let rx = area.right().saturating_sub(rw);
        let hovered = ctx.interaction.hovered(SCOPE);
        let mut rs = if self.scope == Scope::Here {
            t.muted().bg(bg)
        } else {
            t.secondary().bg(bg)
        };
        if hovered {
            rs = rs.fg(t.text_primary).bg(t.lift(bg));
        }
        buf.set_string(rx, area.y, format!(" {readout} "), rs);
        self.scope_area = Rect::new(rx, area.y, rw, 1);
        ctx.clickable(SCOPE, self.scope_area);
        let text_w = rx.saturating_sub(area.x + 3) as usize;
        if self.query.is_empty() {
            let placeholder = match self.group {
                Some(g) => format!("Search {}…", g.to_lowercase()),
                None => "Search actions and resources…".into(),
            };
            buf.set_string(
                area.x + 2,
                area.y,
                truncate(&placeholder, text_w),
                fs.fg(t.text_muted),
            );
        } else {
            let shown = if width(&self.query) > text_w {
                truncate_middle(&self.query, text_w)
            } else {
                self.query.clone()
            };
            let st = if focused {
                fs.add_modifier(Modifier::UNDERLINED)
                    .underline_color(t.accent)
            } else {
                fs
            };
            buf.set_string(area.x + 2, area.y, &shown, st);
        }
        if focused {
            let cx = area.x + 2 + width(&self.query).min(text_w) as u16;
            ctx.set_cursor(Position::new(cx, area.y));
        }
    }

    fn scope_word(&self) -> String {
        match self.scope {
            Scope::Here => "here".into(),
            Scope::Parent => "parent".into(),
            Scope::Children => "children".into(),
            Scope::System => "system".into(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn render_rows(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(FINDER);
        self.list_area = area;
        ctx.control(FINDER, area, false);
        ctx.scrollable(FINDER, area);
        self.explore_areas.clear();
        if self
            .rows
            .iter()
            .all(|r| !matches!(r, Row::Item(..) | Row::Explore(_)))
        {
            let title = if !self.query.trim().is_empty() {
                format!("No matches for “{}”", truncate(&self.query, 30))
            } else {
                match self.scope {
                    Scope::Parent => format!("No project root above {}", w.location.cwd_short()),
                    Scope::Children => {
                        format!("No child projects below {}", w.location.cwd_short())
                    }
                    _ => "Nothing here yet".to_owned(),
                }
            };
            let hint_text = match self.scope {
                Scope::Here => "Esc clears the query · Ctrl+↑ widens to the parent scope",
                Scope::Children => "Esc clears the query · Ctrl+↑ returns to here",
                _ => "Esc clears the query · Ctrl+↓ narrows the scope",
            };
            empty::render(area, buf, t, &EmptyState::new(&title).hint(hint_text), bg);
            return;
        }
        self.scroll.set_content(self.rows.len());
        self.scroll.set_viewport(area.height as usize);
        let has_sb = self.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        // fixed columns over every item row
        let item_rows: Vec<(&Item, &Ranked)> = self
            .rows
            .iter()
            .filter_map(|r| match r {
                Row::Item(i, rk) => Some((&self.items[*i], rk)),
                _ => None,
            })
            .collect();
        let max_label = item_rows
            .iter()
            .map(|(it, _)| width(&it.label))
            .max()
            .unwrap_or(10) as u16;
        let type_w = item_rows
            .iter()
            .map(|(it, _)| width(it.kind.label()))
            .max()
            .unwrap_or(4) as u16;
        let scope_w = item_rows
            .iter()
            .map(|(it, _)| width(&it.scope.word))
            .max()
            .unwrap_or(4) as u16;
        let content_w = row_w.saturating_sub(4);
        let label_w = max_label.min((content_w * 36 / 100).max(14));
        // the safety classes keep a column of their own that never drops:
        // the reason gives way first (CONCEPT §16)
        let risk_w = item_rows
            .iter()
            .filter(|(it, _)| matches!(it.risk, Risk::Destructive | Risk::Privileged))
            .map(|(it, _)| width(it.risk.label()) as u16 + 2)
            .max()
            .unwrap_or(0);
        let fixed = label_w + 2 + type_w + 2 + scope_w + risk_w;
        let reason_w = content_w.saturating_sub(fixed + 2);
        let show_reason = reason_w >= 12;
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let row = Rect::new(area.x, y, row_w, 1);
            match &self.rows[i] {
                Row::Blank => {}
                Row::Heading(h) => heading(buf, area.x + 3, y, row_w.saturating_sub(3), h, t, bg),
                Row::Explore(cells) => {
                    let rid = FINDER.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    let is_cursor = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(
                        row.x,
                        y,
                        t.gutter_symbol(s),
                        t.gutter(s, st.bg.unwrap_or(bg), false),
                    );
                    let mut x = row.x + 3;
                    for (ci, idx) in cells.iter().enumerate() {
                        let it = &self.items[*idx];
                        let cid = EXPLORE_CELL.child(ci);
                        let chosen = is_cursor && ci == self.explore_cursor;
                        let hovered = ctx.interaction.hovered(cid);
                        let text = format!(" {} ", it.label);
                        let cw = width(&text) as u16;
                        if x + cw > row.right() {
                            break;
                        }
                        let mut cs = st.fg(t.text_primary).remove_modifier(Modifier::BOLD);
                        if hovered {
                            cs = cs.bg(t.lift(st.bg.unwrap_or(bg)));
                        }
                        if chosen {
                            cs = cs.add_modifier(Modifier::BOLD);
                            buf.set_string(
                                x,
                                y,
                                "›",
                                st.fg(if s.focused {
                                    t.accent
                                } else {
                                    t.text_secondary
                                }),
                            );
                        }
                        buf.set_string(x + 1, y, &it.label, cs);
                        ctx.clickable(cid, Rect::new(x, y, cw, 1));
                        self.explore_areas.push((cid, Rect::new(x, y, cw, 1), ci));
                        x += cw + 1;
                    }
                    ctx.clickable(rid, row);
                }
                Row::Item(idx, rk) => {
                    let it = &self.items[*idx];
                    let rid = FINDER.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.selected = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    // the query row carries the finder's focus bar; the cursor
                    // row is marked by `›` and the tint alone
                    if s.selected {
                        buf.set_string(
                            row.x + 1,
                            y,
                            "›",
                            st.fg(if focused { t.accent } else { t.text_secondary }),
                        );
                    }
                    let mut x = row.x + 3;
                    // label with matched characters bold
                    let label = truncate(&it.label, label_w as usize);
                    let mut lx = x;
                    for (bi, ch) in label.char_indices() {
                        let mut cs = st;
                        if rk.matched.contains(&bi) {
                            cs = cs.add_modifier(Modifier::BOLD);
                        } else if !s.focused {
                            cs = cs.remove_modifier(Modifier::BOLD);
                        }
                        if it.hidden {
                            cs = cs.fg(t.text_faint);
                        }
                        let g = ch.to_string();
                        buf.set_string(lx, y, &g, cs);
                        lx += width(&g) as u16;
                    }
                    x += label_w + 2;
                    let plain = st.remove_modifier(Modifier::BOLD);
                    buf.set_string(
                        x,
                        y,
                        fit(it.kind.label(), type_w as usize),
                        plain.fg(t.text_muted),
                    );
                    x += type_w + 2;
                    let scope_tone = if it.scope.direction == Scope::Here {
                        t.text_muted
                    } else {
                        t.text_secondary
                    };
                    buf.set_string(
                        x,
                        y,
                        fit(&it.scope.word, scope_w as usize),
                        plain.fg(scope_tone),
                    );
                    x += scope_w + 2;
                    if show_reason {
                        let mut reason = rk.reason.clone();
                        if let Some(tag) = &rk.tag
                            && !reason.starts_with(tag)
                        {
                            reason = format!("{tag} · {reason}");
                        }
                        let rs = if rk.tag.is_some() {
                            plain.fg(t.text_secondary)
                        } else {
                            plain.fg(t.text_muted)
                        };
                        buf.set_string(x, y, truncate_sep(&reason, reason_w as usize), rs);
                    }
                    // only the safety classes take room in the row, as a
                    // plain word: risk changes the gate, not the loudness (D-7)
                    if risk_w > 0 && matches!(it.risk, Risk::Destructive | Risk::Privileged) {
                        let r = it.risk.label();
                        let rx = row.right().saturating_sub(width(r) as u16 + 1);
                        buf.set_string(rx, y, r, plain.fg(t.text_secondary));
                    }
                    ctx.clickable(rid, row);
                }
            }
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(area.right() - 1, area.y, 1, area.height),
                buf,
                ctx,
                FINDER,
                &self.scroll,
                focused,
            );
        }
    }

    fn locate_row(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| FINDER.child(i) == id)
    }

    fn activate_current(&mut self, cx: &mut Cx) -> Outcome {
        let Some(it) = self.current() else {
            return Outcome::Consumed;
        };
        match &it.launch {
            Launch::Group(g) => cx.go(Go::Push(Page::Finder { group: Some(g) })),
            Launch::Scope(s) => cx.go(Go::Scope(*s)),
            _ => cx.go(Go::Run {
                item: it.id.clone(),
                args: vec![],
            }),
        }
        Outcome::Changed
    }

    fn alternatives(&mut self, cx: &mut Cx) -> Outcome {
        let Some(it) = self.current() else {
            return Outcome::Consumed;
        };
        let anchor = self
            .scroll
            .visible_range()
            .find(|&i| i == self.cursor)
            .map(|i| {
                Rect::new(
                    self.list_area.x + 6,
                    self.list_area.y + (i - self.scroll.offset) as u16,
                    1,
                    1,
                )
            })
            .unwrap_or(self.list_area);
        cx.go(Go::Alternatives {
            item: it.id.clone(),
            anchor,
        });
        Outcome::Changed
    }
}

impl Screen for FinderPage {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.dirty {
            self.rebuild(w);
        }
        if cx.focus.is(PREVIEW) {
            return match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.preview_scroll.scroll_by(-1);
                    Outcome::Changed
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.preview_scroll.scroll_by(1);
                    Outcome::Changed
                }
                KeyCode::Char('y') => {
                    if let Some(it) = self.current() {
                        let cmd = it.commands.join("\n");
                        if cmd.is_empty() {
                            cx.status("Nothing to copy: this row has no command");
                        } else {
                            cx.copy(cmd);
                        }
                    }
                    Outcome::Changed
                }
                KeyCode::Esc => {
                    self.drawer = false;
                    cx.focus.focus(FINDER);
                    Outcome::Changed
                }
                KeyCode::Enter => self.activate_current(cx),
                _ => Outcome::Ignored,
            };
        }
        if !cx.focus.is(FINDER) {
            return Outcome::Ignored;
        }
        if key.alt() && key.code == KeyCode::Enter {
            return self.alternatives(cx);
        }
        if key.ctrl() {
            return match key.code {
                KeyCode::Up => {
                    cx.go(Go::Scope(self.scope.outward()));
                    Outcome::Changed
                }
                KeyCode::Down => {
                    cx.go(Go::Scope(self.scope.inward()));
                    Outcome::Changed
                }
                KeyCode::Char('u') => {
                    self.set_query(String::new());
                    Outcome::Changed
                }
                _ => Outcome::Ignored,
            };
        }
        match key.code {
            KeyCode::Up => {
                self.step(-1);
                Outcome::Changed
            }
            KeyCode::Down => {
                self.step(1);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                for _ in 0..self.scroll.viewport_len.max(1) {
                    self.step(-1);
                }
                Outcome::Changed
            }
            KeyCode::PageDown => {
                for _ in 0..self.scroll.viewport_len.max(1) {
                    self.step(1);
                }
                Outcome::Changed
            }
            KeyCode::Home => {
                self.cursor = self.first_selectable();
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::End => {
                self.cursor = self
                    .rows
                    .iter()
                    .rposition(|r| matches!(r, Row::Item(..) | Row::Explore(_)))
                    .unwrap_or(0);
                self.scroll.ensure_visible(self.cursor);
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Right
                if matches!(self.rows.get(self.cursor), Some(Row::Explore(_))) =>
            {
                let n = match &self.rows[self.cursor] {
                    Row::Explore(c) => c.len(),
                    _ => 0,
                };
                if key.code == KeyCode::Left {
                    self.explore_cursor = self.explore_cursor.saturating_sub(1);
                } else {
                    self.explore_cursor = (self.explore_cursor + 1).min(n.saturating_sub(1));
                }
                Outcome::Changed
            }
            KeyCode::Enter => self.activate_current(cx),
            KeyCode::Backspace => {
                if self.query.pop().is_some() {
                    self.dirty = true;
                    Outcome::Changed
                } else {
                    Outcome::Consumed
                }
            }
            KeyCode::Esc => {
                if !self.query.is_empty() {
                    self.set_query(String::new());
                    return Outcome::Changed;
                }
                if self.scope != Scope::Here {
                    cx.go(Go::Scope(Scope::Here));
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            KeyCode::Char(c) if !key.ctrl() && !key.alt() => {
                let mut q = self.query.clone();
                q.push(c);
                self.set_query(q);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.dirty {
            self.rebuild(w);
        }
        if id == SCOPE {
            let next = match self.scope {
                Scope::System => Scope::Children,
                s => s.outward(),
            };
            cx.go(Go::Scope(next));
            return Outcome::Changed;
        }
        if id == PREVIEW || id == scrollbar::id_for(PREVIEW) {
            cx.focus.focus(PREVIEW);
            return Outcome::Changed;
        }
        if let Some((_, _, ci)) = self.explore_areas.iter().find(|(cid, _, _)| *cid == id) {
            if let Some(r) = self.rows.iter().position(|r| matches!(r, Row::Explore(_))) {
                self.cursor = r;
                self.explore_cursor = *ci;
            }
            cx.focus.focus(FINDER);
            return self.activate_current(cx);
        }
        if let Some(i) = self.locate_row(id) {
            if matches!(self.rows[i], Row::Item(..) | Row::Explore(_)) {
                self.cursor = i;
                self.preview_scroll.jump_start();
            }
            cx.focus.focus(FINDER);
            return Outcome::Changed;
        }
        if id == FINDER {
            cx.focus.focus(FINDER);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_double_click(
        &mut self,
        id: WidgetId,
        _pos: Position,
        _w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        if let Some(i) = self.locate_row(id)
            && matches!(self.rows[i], Row::Item(..))
        {
            self.cursor = i;
            return self.activate_current(cx);
        }
        Outcome::Ignored
    }

    fn on_secondary(
        &mut self,
        id: WidgetId,
        _pos: Position,
        _w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        if let Some(i) = self.locate_row(id)
            && matches!(self.rows[i], Row::Item(..))
        {
            self.cursor = i;
            cx.focus.focus(FINDER);
            return self.alternatives(cx);
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == PREVIEW {
            self.preview_scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if id == FINDER {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        if w.tick != self.last_tick {
            self.last_tick = w.tick;
            self.dirty = true;
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn enter(&mut self, w: &mut World, _cx: &mut Cx) {
        self.rebuild(w);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        if self.dirty {
            self.rebuild(w);
        }
        let t = ctx.theme;
        let query = Rect::new(area.x, area.y, area.width, 1);
        self.render_query(query, buf, ctx);
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2),
        );
        self.split = area.width >= SPLIT_MIN;
        if self.split {
            let pw = (area.width * 34 / 100).clamp(34, 48);
            let list = Rect::new(
                body.x,
                body.y,
                body.width.saturating_sub(pw + 2),
                body.height,
            );
            let preview = Rect::new(list.right() + 2, body.y, pw, body.height);
            self.render_rows(list, buf, ctx, w);
            self.render_preview(preview, buf, ctx, w);
        } else {
            let preview_focused = ctx.interaction.focused(PREVIEW);
            if preview_focused || self.drawer {
                self.drawer = true;
                // the list keeps its focus stop so Tab still reaches it
                ctx.control(FINDER, Rect::ZERO, false);
                self.render_preview(body, buf, ctx, w);
            } else {
                let summary_h = 1u16;
                let list = Rect::new(
                    body.x,
                    body.y,
                    body.width,
                    body.height.saturating_sub(summary_h + 1),
                );
                self.render_rows(list, buf, ctx, w);
                // a one-line summary stands in for the preview
                if let Some(it) = self.current() {
                    let cmd = it
                        .commands
                        .first()
                        .cloned()
                        .unwrap_or_else(|| "opens inside holla".into());
                    // the command gives way in the middle; scope and risk
                    // stay readable at every width
                    let tail = format!(" · {} · {}", it.scope.word, it.risk.label());
                    let room =
                        (body.width.saturating_sub(3) as usize).saturating_sub(2 + width(&tail));
                    let line = format!("→ {}{tail}", truncate_middle(&cmd, room.max(8)));
                    let y = body.bottom().saturating_sub(1);
                    buf.set_string(
                        body.x + 3,
                        y,
                        truncate(&line, body.width.saturating_sub(3) as usize),
                        t.muted(),
                    );
                }
                ctx.control(PREVIEW, Rect::ZERO, false);
            }
        }
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus == Some(PREVIEW) {
            return vec![
                hint("↑↓", "Scroll"),
                hint("y", "Copy command"),
                hint("Enter", "Run"),
                hint("Esc", "Results"),
            ];
        }
        let mut v = vec![
            hint("Type", "Search"),
            hint("↑↓", "Move"),
            self.primary_hint(),
            hint("Alt+Enter", "More"),
            hint("Tab", "Preview"),
            hint("Ctrl+↑↓", "Scope"),
        ];
        v.push(if !self.query.is_empty() {
            hint("Esc", "Clear")
        } else if self.scope != Scope::Here {
            hint("Esc", "Here")
        } else if self.group.is_some() {
            hint("Esc", "Back")
        } else {
            hint("Ctrl+G", "Activities")
        });
        v
    }

    fn crumb(&self, _w: &World) -> String {
        match self.group {
            Some(g) => g.to_owned(),
            None => "Here".into(),
        }
    }

    fn status(&self, w: &World) -> StatusBits {
        let mut bits = StatusBits::default();
        if w.discovering() {
            let pending = w.pending_sources();
            let shown: Vec<&str> = pending.iter().copied().take(3).collect();
            bits.center = Some(
                StatusItem::new(
                    format!(
                        "discovering {}{} · {} of {}",
                        shown.join(", "),
                        if pending.len() > 3 { ", …" } else { "" },
                        w.sources_done(),
                        w.sources.len()
                    ),
                    Tone::Secondary,
                )
                .busy()
                .priority(6),
            );
        } else {
            let failed = w.failed_sources();
            if !failed.is_empty() {
                let names: Vec<&str> = failed.iter().map(|(n, _)| *n).collect();
                bits.center = Some(
                    StatusItem::new(format!("! {} unavailable", names.join(", ")), Tone::Error)
                        .priority(7),
                );
            } else if self.scope != Scope::Here {
                bits.center = Some(
                    StatusItem::new(
                        format!("scope {}", self.scope_title(w).to_lowercase()),
                        Tone::Secondary,
                    )
                    .priority(6),
                );
            } else if self.group.is_some() {
                bits.center = Some(StatusItem::new("discovery complete", Tone::Muted).priority(3));
            }
        }
        bits
    }

    fn typing_hot(&self) -> bool {
        true
    }

    fn is_editing(&self) -> bool {
        false
    }

    fn animating(&self, w: &World) -> bool {
        w.discovering() || w.live_activities() > 0
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(FINDER)
    }

    fn as_finder(&mut self) -> Option<&mut FinderPage> {
        Some(self)
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        if self.group.is_some() {
            cx.go(Go::Pop);
            Outcome::Changed
        } else {
            cx.go(Go::Quit);
            Outcome::Changed
        }
    }
}

/// Shared helper for pages that show an item's facts.
pub fn item_facts(it: &Item, w: &World, theme: &Theme) -> Vec<junie_tui::widgets::props::Prop> {
    use junie_tui::widgets::props::Prop;
    let _ = theme;
    let mut v = vec![
        Prop::new("Action", it.summary.clone()).wrap(),
        Prop::new("Host", format!("{} · {}", w.host.name, w.host.role.label())),
        Prop::new(
            "In",
            if it.scope.direction == Scope::System {
                format!("{} · host-wide", w.host.name)
            } else {
                w.location.short(&it.scope.runs_in)
            },
        ),
        Prop::new(
            "Scope",
            if it.scope.direction == Scope::System || it.scope.defined_at == it.scope.runs_in {
                it.scope.word.clone()
            } else {
                format!(
                    "{} · {}",
                    it.scope.word,
                    w.location.short(&it.scope.defined_at)
                )
            },
        ),
    ];
    if !it.effects.is_empty() {
        v.push(Prop::new("Changes", it.effects.join(" · ")).wrap());
    }
    v.push(Prop::new("Risk", it.risk.label()).tone(risk_tone(it.risk)));
    v.push(Prop::new("Data", it.freshness.label()).tone(Tone::Muted));
    v
}
