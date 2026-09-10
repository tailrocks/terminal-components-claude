//! Disk usage and cleanup: progressive largest-first analysis of a path
//! with the Mole-informed hierarchy (filesystems, largest here, rebuildable
//! artifacts by project, system cleanup families), freshness-aware default
//! selection, candidate facts, and a route into an editable cleanup plan.

use std::collections::BTreeSet;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width, wrap};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{
    Meter, MeterTone, MeterVisual, ProgressStatus, render_bar, render_spinner,
};
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::stack::{Candidate, Family};
use crate::screens::{Cx, Go, Screen, StatusBits, heading, plural};
use crate::sim::world::World;

pub const TREE: WidgetId = WidgetId::of("disk.tree");
pub const DETAIL: WidgetId = WidgetId::of("disk.detail");

#[derive(Debug, Clone, PartialEq)]
enum Row {
    Heading(String),
    Blank,
    Fs(usize),
    Large(usize),
    Candidate(usize),
}

pub struct DiskPage {
    pub path: String,
    rows: Vec<Row>,
    pub cursor: usize,
    scroll: ScrollState,
    detail_scroll: ScrollState,
    pub selected: BTreeSet<String>,
    seeded: bool,
    drawer: bool,
    last_tick: u64,
    /// Short bodies drop the Filesystems section: the status bar keeps the
    /// root meter, and the candidates are what the page is for.
    compact: bool,
}

impl DiskPage {
    pub fn new(path: &str) -> Self {
        Self {
            path: path.into(),
            rows: vec![],
            cursor: 0,
            scroll: ScrollState::default(),
            detail_scroll: ScrollState::default(),
            selected: BTreeSet::new(),
            seeded: false,
            drawer: false,
            last_tick: u64::MAX,
            compact: false,
        }
    }

    fn progress(&self, w: &World) -> u64 {
        w.scan_progress()
            .map(|(done, _)| done)
            .unwrap_or(w.disk.scan_ticks)
    }

    fn visible_candidates<'a>(&self, w: &'a World) -> Vec<(usize, &'a Candidate)> {
        let p = self.progress(w);
        w.disk
            .candidates
            .iter()
            .enumerate()
            .filter(|(_, c)| c.found_at <= p)
            .collect()
    }

    fn rebuild(&mut self, w: &World) {
        let keep = match self.rows.get(self.cursor) {
            Some(Row::Candidate(i)) => {
                Some(self.rows[self.cursor].clone()).map(|_| w.disk.candidates[*i].path.clone())
            }
            _ => None,
        };
        let p = self.progress(w);
        if !self.seeded {
            for c in &w.disk.candidates {
                if c.default_selected() {
                    self.selected.insert(c.path.clone());
                }
            }
            self.seeded = true;
        }
        let mut rows = vec![];
        if !self.compact {
            rows.push(Row::Heading("Filesystems".into()));
            for i in 0..w.disk.filesystems.len() {
                rows.push(Row::Fs(i));
            }
        }
        let large: Vec<usize> = {
            let mut v: Vec<usize> = (0..w.disk.large.len())
                .filter(|&i| w.disk.large[i].found_at <= p)
                .collect();
            v.sort_by(|a, b| {
                w.disk.large[*b]
                    .gb
                    .partial_cmp(&w.disk.large[*a].gb)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            v
        };
        if !large.is_empty() {
            if !rows.is_empty() {
                rows.push(Row::Blank);
            }
            // the scan walks the whole filesystem the folder sits on
            rows.push(Row::Heading(format!(
                "Largest on {}",
                w.disk
                    .root_fs()
                    .map(|f| f.mount.as_str())
                    .unwrap_or("this disk")
            )));
            for i in large {
                rows.push(Row::Large(i));
            }
        }
        let cands = self.visible_candidates(w);
        let artifacts: Vec<usize> = cands
            .iter()
            .filter(|(_, c)| c.family == Family::ProjectArtifacts)
            .map(|(i, _)| *i)
            .collect();
        if !artifacts.is_empty() {
            rows.push(Row::Blank);
            rows.push(Row::Heading("Rebuildable artifacts · by project".into()));
            let mut sorted = artifacts;
            sorted.sort_by(|a, b| {
                w.disk.candidates[*a]
                    .project
                    .cmp(&w.disk.candidates[*b].project)
            });
            for i in sorted {
                rows.push(Row::Candidate(i));
            }
        }
        let families = [
            Family::DeveloperCaches,
            Family::Containers,
            Family::ApplicationCaches,
            Family::PackageCaches,
            Family::Logs,
            Family::Temp,
            Family::LargeFiles,
        ];
        let mut any = false;
        for f in families {
            let in_family: Vec<usize> = cands
                .iter()
                .filter(|(_, c)| c.family == f)
                .map(|(i, _)| *i)
                .collect();
            if in_family.is_empty() {
                continue;
            }
            if !any {
                rows.push(Row::Blank);
                rows.push(Row::Heading("System cleanup families".into()));
                any = true;
            }
            let gb: f32 = in_family.iter().map(|&i| w.disk.candidates[i].gb).sum();
            rows.push(Row::Heading(format!(
                "  {} · {} · {gb:.1} GB",
                f.label(),
                plural(in_family.len(), "candidate", "candidates")
            )));
            for i in in_family {
                rows.push(Row::Candidate(i));
            }
        }
        self.rows = rows;
        let target = keep.and_then(|path| {
            self.rows
                .iter()
                .position(|r| matches!(r, Row::Candidate(i) if w.disk.candidates[*i].path == path))
        });
        self.cursor = target.unwrap_or_else(|| self.cursor.min(self.rows.len().saturating_sub(1)));
        if !matches!(
            self.rows.get(self.cursor),
            Some(Row::Candidate(_) | Row::Large(_) | Row::Fs(_))
        ) {
            self.cursor = self
                .rows
                .iter()
                .position(|r| matches!(r, Row::Candidate(_) | Row::Large(_)))
                .unwrap_or(self.cursor);
        }
        self.scroll.set_content(self.rows.len());
    }

    fn step(&mut self, delta: isize) {
        let n = self.rows.len() as isize;
        let mut c = self.cursor as isize;
        loop {
            c += delta;
            if c < 0 || c >= n {
                return;
            }
            if matches!(
                self.rows[c as usize],
                Row::Candidate(_) | Row::Large(_) | Row::Fs(_)
            ) {
                self.cursor = c as usize;
                self.detail_scroll.jump_start();
                self.scroll.ensure_visible(self.cursor);
                return;
            }
        }
    }

    fn selected_gb(&self, w: &World) -> f32 {
        w.disk
            .candidates
            .iter()
            .filter(|c| self.selected.contains(&c.path))
            .map(|c| c.gb)
            .sum()
    }

    fn detail_props(&self, w: &World) -> (String, String, Vec<Prop>) {
        match self.rows.get(self.cursor) {
            Some(Row::Candidate(i)) => {
                let c = &w.disk.candidates[*i];
                let mut v = vec![
                    Prop::new("Path", w.location.short(&c.path)).wrap(),
                    Prop::new("Family", c.family.label()),
                    Prop::new("Project", c.project.clone().unwrap_or("–".into())),
                    Prop::new("Size", format!("{:.1} GB · {} items", c.gb, c.items)),
                    Prop::new("Activity", c.age_label()).tone(if c.inactive_days.is_none() {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    }),
                    Prop::new("Why", c.why.clone()).wrap(),
                    Prop::new("Regenerate", c.regenerate.clone())
                        .tone(Tone::Secondary)
                        .wrap(),
                    Prop::new("Method", c.method.label()),
                    Prop::new("Confidence", c.confidence.label()).tone(match c.confidence {
                        crate::domain::stack::Confidence::Unverifiable => Tone::Warning,
                        _ => Tone::Normal,
                    }),
                ];
                if let Some(p) = &c.active_process {
                    v.push(Prop::new("In use by", p.clone()).tone(Tone::Error));
                }
                if let Some(p) = &c.privilege {
                    v.push(Prop::new("Privilege", p.clone()).tone(Tone::Warning));
                }
                if let Some(s) = &c.shares_with {
                    v.push(Prop::new(
                        "Shares with",
                        format!("{s} · cleaned in order, never in parallel"),
                    ));
                }
                if let Some(r) = c.skip_reason() {
                    v.push(Prop::new("Skipped", r).tone(Tone::Error));
                } else {
                    v.push(
                        Prop::new(
                            "Default",
                            if c.default_selected() {
                                "selected · stale and confidently rebuildable"
                            } else {
                                "unselected · recent or uncertain"
                            },
                        )
                        .tone(Tone::Muted),
                    );
                }
                (
                    c.path.rsplit('/').next().unwrap_or("").to_owned(),
                    "candidate".into(),
                    v,
                )
            }
            Some(Row::Large(i)) => {
                let l = &w.disk.large[*i];
                (
                    w.location.short(&l.path),
                    l.kind.to_owned(),
                    vec![
                        Prop::new("Size", format!("{:.1} GB", l.gb)),
                        Prop::new(
                            "Note",
                            "observation only · nothing is deleted from this view",
                        )
                        .tone(Tone::Muted)
                        .wrap(),
                    ],
                )
            }
            Some(Row::Fs(i)) => {
                let f = &w.disk.filesystems[*i];
                (
                    f.mount.clone(),
                    "filesystem".into(),
                    vec![
                        Prop::new(
                            "Used",
                            format!("{} of {} GB · {}%", f.used_gb, f.total_gb, f.pct()),
                        ),
                        Prop::new("Free", format!("{} GB", f.total_gb - f.used_gb)),
                    ],
                )
            }
            _ => ("Disk".into(), String::new(), vec![]),
        }
    }

    fn render_detail(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(DETAIL);
        let (title, meta, props) = self.detail_props(w);
        let panel = Panel::card(Some(&title)).focused(focused).meta(&meta);
        let bg = panel.bg(t);
        let inner = panel.render(area, buf, t);
        ctx.control(DETAIL, area, false);
        ctx.scrollable(DETAIL, inner);
        let label_w = props.iter().map(|p| width(&p.label)).max().unwrap_or(4) as u16 + 2;
        let vw = inner.width.saturating_sub(label_w) as usize;
        let mut flat: Vec<(String, String, Tone)> = vec![];
        for p in props {
            for (i, part) in wrap(&p.value, vw.max(8)).into_iter().enumerate() {
                flat.push((
                    if i == 0 {
                        p.label.clone()
                    } else {
                        String::new()
                    },
                    part,
                    p.tone,
                ));
            }
        }
        self.detail_scroll.set_content(flat.len());
        self.detail_scroll.set_viewport(inner.height as usize);
        for (k, i) in self.detail_scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let (l, v, tone) = &flat[i];
            buf.set_string(inner.x, y, l, t.muted().bg(bg));
            buf.set_string(
                inner.x + label_w,
                y,
                truncate(v, vw),
                Style::new().fg(t.tone(*tone)).bg(bg),
            );
        }
        if self.detail_scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                DETAIL,
                &self.detail_scroll,
                focused,
            );
        }
    }

    fn render_tree(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(TREE);
        self.scroll.set_content(self.rows.len());
        self.scroll.set_viewport(area.height as usize);
        ctx.control(TREE, area, false);
        ctx.scrollable(TREE, area);
        let has_sb = self.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        let label_w = (row_w * 40 / 100).clamp(18, 44);
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let row = Rect::new(area.x, y, row_w, 1);
            match &self.rows[i] {
                Row::Blank => {}
                Row::Heading(h) => heading(buf, area.x + 3, y, row_w.saturating_sub(3), h, t, bg),
                Row::Fs(fi) => {
                    let f = &w.disk.filesystems[*fi];
                    let rid = TREE.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.selected = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
                    if s.selected {
                        buf.set_string(
                            row.x + 1,
                            y,
                            "›",
                            st.fg(if focused { t.accent } else { t.text_secondary }),
                        );
                    }
                    buf.set_string(row.x + 3, y, fit(&f.mount, 22), st);
                    let mx = row.x + 27;
                    let mw = 24u16.min(row_w.saturating_sub(29));
                    if mw >= 8 {
                        Meter::new(Some(f.pct()))
                            .value(format!("{}%", f.pct()))
                            .tone(MeterTone::Normal)
                            .visual(MeterVisual::Line)
                            .render(Rect::new(mx, y, mw, 1), buf, ctx, st.bg.unwrap_or(bg));
                        let text = format!("{} of {} GB", f.used_gb, f.total_gb);
                        if mx + mw + 2 + width(&text) as u16 <= row.right() {
                            buf.set_string(
                                mx + mw + 2,
                                y,
                                &text,
                                st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                            );
                        }
                    }
                    ctx.clickable(rid, row);
                }
                Row::Large(li) => {
                    let l = &w.disk.large[*li];
                    let rid = TREE.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.selected = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
                    if s.selected {
                        buf.set_string(
                            row.x + 1,
                            y,
                            "›",
                            st.fg(if focused { t.accent } else { t.text_secondary }),
                        );
                    }
                    let plain = st.remove_modifier(Modifier::BOLD);
                    buf.set_string(
                        row.x + 3,
                        y,
                        fit(&w.location.short(&l.path), label_w as usize + 8),
                        st,
                    );
                    let x = row.x + 3 + label_w + 10;
                    if x + 10 <= row.right() {
                        buf.set_string(
                            x,
                            y,
                            format!("{:>7.1} GB", l.gb),
                            plain.fg(t.text_secondary),
                        );
                    }
                    if x + 24 <= row.right() {
                        buf.set_string(
                            x + 12,
                            y,
                            truncate(l.kind, row.right().saturating_sub(x + 13) as usize),
                            plain.fg(t.text_muted),
                        );
                    }
                    ctx.clickable(rid, row);
                }
                Row::Candidate(ci) => {
                    let c = &w.disk.candidates[*ci];
                    let rid = TREE.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.selected = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(row.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
                    let plain = st.remove_modifier(Modifier::BOLD);
                    let skip = c.skip_reason();
                    let checked = self.selected.contains(&c.path);
                    let quiet = if s.selected {
                        t.text_muted
                    } else {
                        t.text_faint
                    };
                    let (mark, ms) = if skip.is_some() {
                        ("[ ]", plain.fg(quiet))
                    } else if checked {
                        ("[✓]", plain.fg(t.accent))
                    } else {
                        ("[ ]", plain.fg(t.text_muted))
                    };
                    buf.set_string(row.x + 1, y, mark, ms);
                    let label = match &c.project {
                        Some(p) if c.family == Family::ProjectArtifacts => {
                            format!("{p} › {}", c.path.rsplit('/').next().unwrap_or(""))
                        }
                        _ => w.location.short(&c.path),
                    };
                    let ls = if skip.is_some() { st.fg(quiet) } else { st };
                    buf.set_string(row.x + 5, y, fit(&label, label_w as usize), ls);
                    let mut x = row.x + 5 + label_w + 2;
                    if x + 9 <= row.right() {
                        buf.set_string(
                            x,
                            y,
                            format!("{:>6.1} GB", c.gb),
                            plain.fg(t.text_secondary),
                        );
                    }
                    x += 11;
                    let age = c.age_label();
                    let age_tone = match c.inactive_days {
                        _ if skip.is_some() => quiet,
                        None => t.warning,
                        Some(0) => t.text_secondary,
                        _ => t.text_muted,
                    };
                    if x + 18 <= row.right() {
                        buf.set_string(x, y, fit(&age, 18), plain.fg(age_tone));
                    }
                    x += 20;
                    let method = match &c.method {
                        crate::domain::stack::Method::Trash => "Trash".to_owned(),
                        crate::domain::stack::Method::Permanent => "permanent".to_owned(),
                        crate::domain::stack::Method::Tool(cmd) => {
                            cmd.split_whitespace().next().unwrap_or("tool").to_owned()
                        }
                    };
                    let skipped = skip.is_some();
                    let tail = match skip {
                        Some(r) => r,
                        None => method,
                    };
                    let avail = row.right().saturating_sub(x + 1) as usize;
                    if avail >= 6 {
                        buf.set_string(
                            x,
                            y,
                            truncate(&tail, avail),
                            plain.fg(if skipped { quiet } else { t.text_faint }),
                        );
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
                TREE,
                &self.scroll,
                focused,
            );
        }
    }

    fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| TREE.child(i) == id)
    }
}

impl Screen for DiskPage {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if cx.focus.is(DETAIL) {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.detail_scroll.scroll_by(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    self.detail_scroll.scroll_by(1);
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    self.drawer = false;
                    cx.focus.focus(TREE);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.step(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.step(1);
                Outcome::Changed
            }
            KeyCode::Home | KeyCode::Char('g') => {
                self.cursor = 0;
                self.step(1);
                self.cursor = self.cursor.saturating_sub(0);
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = self.rows.len();
                self.step(-1);
                Outcome::Changed
            }
            KeyCode::Char(' ') => {
                if let Some(Row::Candidate(i)) = self.rows.get(self.cursor) {
                    let c = &w.disk.candidates[*i];
                    if let Some(r) = c.skip_reason() {
                        cx.status(format!(
                            "Cannot select {} · {r}",
                            c.path.rsplit('/').next().unwrap_or("")
                        ));
                    } else if self.selected.remove(&c.path) {
                        cx.status(format!(
                            "Unselected · {:.1} GB selected",
                            self.selected_gb(w) - 0.0
                        ));
                    } else {
                        self.selected.insert(c.path.clone());
                        cx.status(format!(
                            "Selected · {:.1} GB in {}",
                            self.selected_gb(w),
                            plural(self.selected.len(), "target", "targets")
                        ));
                    }
                    return Outcome::Changed;
                }
                cx.status("Only cleanup candidates can be selected");
                Outcome::Changed
            }
            KeyCode::Char('a') => {
                let visible: Vec<&Candidate> = self
                    .visible_candidates(w)
                    .into_iter()
                    .map(|(_, c)| c)
                    .filter(|c| c.skip_reason().is_none())
                    .collect();
                let all = visible.iter().all(|c| self.selected.contains(&c.path));
                for c in visible {
                    if all {
                        self.selected.remove(&c.path);
                    } else {
                        self.selected.insert(c.path.clone());
                    }
                }
                cx.status(if all {
                    "Everything unselected".to_owned()
                } else {
                    format!(
                        "Every eligible candidate selected · {:.1} GB",
                        self.selected_gb(w)
                    )
                });
                Outcome::Changed
            }
            KeyCode::Char('c') => {
                if self.selected.is_empty() {
                    cx.status("Select at least one candidate first · Space selects");
                } else {
                    cx.go(Go::CleanupPlan(self.selected.iter().cloned().collect()));
                }
                Outcome::Changed
            }
            KeyCode::Char('p') => {
                self.drawer = !self.drawer;
                cx.focus.focus(if self.drawer { DETAIL } else { TREE });
                Outcome::Changed
            }
            KeyCode::Enter => {
                cx.focus.focus(DETAIL);
                self.drawer = true;
                Outcome::Changed
            }
            KeyCode::Char('y') => {
                let p = match self.rows.get(self.cursor) {
                    Some(Row::Candidate(i)) => w.disk.candidates[*i].path.clone(),
                    Some(Row::Large(i)) => w.disk.large[*i].path.clone(),
                    Some(Row::Fs(i)) => w.disk.filesystems[*i].mount.clone(),
                    _ => String::new(),
                };
                if !p.is_empty() {
                    cx.copy(p);
                }
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if id == DETAIL {
            cx.focus.focus(DETAIL);
            return Outcome::Changed;
        }
        if let Some(i) = self.locate(id) {
            self.cursor = i;
            cx.focus.focus(TREE);
            if let Some(Row::Candidate(ci)) = self.rows.get(i) {
                let c = &w.disk.candidates[*ci];
                if c.skip_reason().is_none() && !self.selected.remove(&c.path) {
                    self.selected.insert(c.path.clone());
                }
            }
            return Outcome::Changed;
        }
        if id == TREE {
            cx.focus.focus(TREE);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == TREE {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if id == DETAIL {
            self.detail_scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        if w.tick != self.last_tick {
            self.last_tick = w.tick;
            self.rebuild(w);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn enter(&mut self, w: &mut World, _cx: &mut Cx) {
        if w.scan_started.is_none() {
            w.scan_started = Some(w.tick);
        }
        self.rebuild(w);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let compact = area.height < 22;
        if compact != self.compact {
            self.compact = compact;
            self.last_tick = u64::MAX;
        }
        self.rebuild(w);
        let t = ctx.theme;
        let title = format!("Disk usage · {}", w.location.short(&self.path));
        buf.set_string(area.x + 1, area.y, &title, t.title());
        // scan progress or completion line
        let (done, total) = w
            .scan_progress()
            .unwrap_or((w.disk.scan_ticks, w.disk.scan_ticks));
        let cands = self.visible_candidates(w);
        let found_gb: f32 = cands.iter().map(|(_, c)| c.gb).sum();
        let y1 = area.y + 1;
        if done < total {
            let label = format!(
                "scanning · {} so far",
                plural(cands.len(), "candidate", "candidates")
            );
            render_spinner(
                Rect::new(area.x + 1, y1, area.width.saturating_sub(2), 1),
                buf,
                ctx,
                &label,
                t.canvas,
            );
            let bx = area.x + 4 + width(&label) as u16;
            render_bar(
                Rect::new(bx, y1, 32.min(area.width.saturating_sub(bx + 2)), 1),
                buf,
                ctx,
                "",
                done as f64 / total.max(1) as f64,
                ProgressStatus::Active,
                t.canvas,
            );
        } else {
            let mut line = format!(
                "scan complete · {found_gb:.1} GB in {} · {:.1} GB reclaimable · {} selected ({:.1} GB)",
                plural(cands.len(), "candidate", "candidates"),
                w.disk.reclaimable_gb(),
                self.selected.len(),
                self.selected_gb(w)
            );
            if let Some(r) = &w.disk.partial_reason {
                line = format!("▲ partial · {r} · {line}");
            }
            let tone = if w.disk.partial_reason.is_some() {
                t.warning
            } else {
                t.text_muted
            };
            buf.set_string(
                area.x + 1,
                y1,
                truncate(&line, area.width.saturating_sub(2) as usize),
                Style::new().fg(tone),
            );
        }
        let body = Rect::new(
            area.x,
            area.y + 3,
            area.width,
            area.height.saturating_sub(3),
        );
        let split = area.width >= crate::screens::finder::SPLIT_MIN;
        if split {
            let dw = (area.width * 38 / 100).clamp(34, 50);
            let tree = Rect::new(
                body.x,
                body.y,
                body.width.saturating_sub(dw + 2),
                body.height,
            );
            let detail = Rect::new(tree.right() + 2, body.y, dw, body.height);
            self.render_tree(tree, buf, ctx, w);
            self.render_detail(detail, buf, ctx, w);
        } else if self.drawer || ctx.interaction.focused(DETAIL) {
            self.drawer = true;
            ctx.control(TREE, Rect::ZERO, false);
            self.render_detail(body, buf, ctx, w);
        } else {
            self.render_tree(body, buf, ctx, w);
            ctx.control(DETAIL, Rect::ZERO, false);
        }
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus == Some(DETAIL) {
            return vec![hint("↑↓", "Scroll"), hint("Esc", "List")];
        }
        let on_candidate = matches!(self.rows.get(self.cursor), Some(Row::Candidate(_)));
        let mut v = vec![hint("↑↓", "Move")];
        if on_candidate {
            v.push(hint("Space", "Select"));
        }
        v.push(hint("a", "All"));
        v.push(hint("c", "Cleanup plan"));
        v.push(hint("Enter", "Facts"));
        v.push(hint("y", "Copy path"));
        v.push(hint("Esc", "Back"));
        v
    }

    fn crumb(&self, _w: &World) -> String {
        "Disk › Usage".into()
    }

    fn status(&self, w: &World) -> StatusBits {
        let (done, total) = w.scan_progress().unwrap_or((1, 1));
        let text = if done < total {
            format!(
                "scanning {} · {}%",
                w.location.short(&self.path),
                (done as f64 / total.max(1) as f64 * 100.0).round() as u64
            )
        } else {
            format!(
                "{} selected · {:.1} GB",
                self.selected.len(),
                self.selected_gb(w)
            )
        };
        let mut item = StatusItem::new(text, Tone::Secondary).priority(6);
        if done < total {
            item = item.busy();
        }
        StatusBits {
            center: Some(item),
            right: vec![],
        }
    }

    fn animating(&self, w: &World) -> bool {
        w.scan_progress().is_some_and(|(d, t)| d < t)
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(TREE)
    }
}
