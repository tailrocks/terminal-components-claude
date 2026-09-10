//! Cleanup review (HP20–HP22): every detected insight category with its
//! candidates, the shared eligibility rule, arbitrary selection, the
//! Trash/permanent mode and dry-run toggles, and the route into the
//! authorized deletion gate. The same page serves Gradle and IntelliJ
//! recursive cleanups through the shared candidate walker.

use std::collections::BTreeSet;

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width, wrap};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use crate::domain::catalog::{InsightCategory, insight_candidates, observe_process};
use crate::domain::cleanup::{self, DeleteItem, DeletePlan, Eligibility, Mode, ProcessObservation};
use crate::screens::{Cx, Go, Page, Screen, StatusBits, heading, plural};
use crate::sim::fs::human;
use crate::sim::world::World;

pub const LIST: WidgetId = WidgetId::of("cleanup.list");
pub const DETAIL: WidgetId = WidgetId::of("cleanup.detail");

#[derive(Debug, Clone, PartialEq, Eq)]
enum Row {
    Heading(String),
    Blank,
    Category(usize),
    Candidate(usize, usize),
    Note(String),
}

/// A recursive tool cleanup (Gradle, IDEA) modelled as one synthetic
/// category over the shared walker.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Special {
    id: String,
    label: String,
    note: String,
    prerequisite: Option<String>,
}

pub struct CleanupPage {
    /// Filter: one category id, or a special recursive cleanup.
    pub filter: Option<String>,
    cats: Vec<InsightCategory>,
    special: Option<Special>,
    rows: Vec<Row>,
    pub cursor: usize,
    scroll: ScrollState,
    detail_scroll: ScrollState,
    /// Checked candidate paths.
    pub checked: BTreeSet<String>,
    /// Categories drilled open.
    open: BTreeSet<usize>,
    pub mode: Mode,
    pub dry_run: bool,
    drawer: bool,
    seeded: bool,
    last_tick: u64,
    /// The last built plan revision, to show drift.
    pub reviewed: Option<u64>,
}

impl CleanupPage {
    pub fn new(filter: Option<&str>) -> Self {
        Self {
            filter: filter.map(str::to_owned),
            cats: vec![],
            special: None,
            rows: vec![],
            cursor: 0,
            scroll: ScrollState::default(),
            detail_scroll: ScrollState::default(),
            checked: BTreeSet::new(),
            open: BTreeSet::new(),
            mode: Mode::Trash,
            dry_run: false,
            drawer: false,
            seeded: false,
            last_tick: u64::MAX,
            reviewed: None,
        }
    }

    fn special_category(&self, w: &World) -> Option<(Special, InsightCategory)> {
        let id = self.filter.as_deref()?;
        let cwd = w.location.cwd.clone();
        let (special, (paths, unreadable)): (Special, (Vec<String>, Vec<String>)) = match id {
            "gradle.clean-all" => (
                Special {
                    id: id.into(),
                    label: "Gradle outputs".into(),
                    note: ".gradle and build directories below this folder · depth 5 · no symlinks · no node_modules".into(),
                    prerequisite: Some("gradle --stop".into()),
                },
                cleanup::walk_candidates_report(&w.fs, &cwd, &|p, is_dir| is_dir && (p.ends_with("/.gradle") || p.ends_with("/build"))),
            ),
            "idea.clean" => (
                Special {
                    id: id.into(),
                    label: "IntelliJ metadata".into(),
                    note: ".idea directories and lowercase .iml files below this folder · depth 5 · no symlinks · no node_modules".into(),
                    prerequisite: None,
                },
                cleanup::walk_candidates_report(&w.fs, &cwd, &|p, is_dir| (is_dir && p.ends_with("/.idea")) || (!is_dir && p.ends_with(".iml"))),
            ),
            _ => return None,
        };
        let cat: &'static cleanup::Category = cleanup::category("project.artifacts").unwrap();
        let mut candidates = vec![];
        for p in paths {
            let scan = w.fs.scan(
                &p,
                &crate::sim::fs::ScanOptions {
                    include_hidden: true,
                    ..Default::default()
                },
            );
            let age =
                w.fs.get(&p)
                    .and_then(|n| cleanup::age_days(n.mtime, w.now_secs()));
            candidates.push(crate::domain::catalog::InsightCandidate {
                path: p,
                bytes: scan.allocated,
                partial: scan.errors() > 0,
                age_days: age,
                eligibility: Eligibility::Preselected,
            });
        }
        candidates.sort_by(|a, b| a.path.cmp(&b.path));
        Some((
            special,
            InsightCategory {
                category: cat,
                candidates,
                process: ProcessObservation::NotRunning,
                unreadable,
            },
        ))
    }

    fn rebuild(&mut self, w: &World) {
        let keep = match self.rows.get(self.cursor) {
            Some(Row::Candidate(c, i)) => self
                .cats
                .get(*c)
                .and_then(|k| k.candidates.get(*i))
                .map(|k| k.path.clone()),
            _ => None,
        };
        if let Some((sp, cat)) = self.special_category(w) {
            self.cats = vec![cat];
            self.special = Some(sp);
        } else {
            self.cats = insight_candidates(w);
            self.special = None;
            if let Some(f) = &self.filter {
                self.cats.retain(|c| c.category.id == f);
            }
        }
        // the checked set follows candidate identity: vanished paths drop,
        // eligibility is re-checked every rebuild
        let live: BTreeSet<&str> = self
            .cats
            .iter()
            .flat_map(|c| c.candidates.iter())
            .filter(|k| k.eligibility.selectable())
            .map(|k| k.path.as_str())
            .collect();
        self.checked.retain(|p| live.contains(p.as_str()));
        if !self.seeded {
            for c in &self.cats {
                for k in &c.candidates {
                    if k.eligibility == Eligibility::Preselected {
                        self.checked.insert(k.path.clone());
                    }
                }
            }
            if self.filter.is_some() {
                for i in 0..self.cats.len() {
                    self.open.insert(i);
                }
            }
            self.seeded = true;
        }
        let mut rows = vec![];
        if self.cats.is_empty() {
            rows.push(Row::Note(match &self.filter {
                Some(f) => format!("{f} is not detected on this host"),
                None => "no cleanup categories are detected on this host".into(),
            }));
        }
        for (ci, c) in self.cats.iter().enumerate() {
            if ci > 0 {
                rows.push(Row::Blank);
            }
            rows.push(Row::Category(ci));
            if self.open.contains(&ci) {
                if c.candidates.is_empty() {
                    rows.push(Row::Note("  no candidates · nothing to remove".into()));
                }
                for (ki, _) in c.candidates.iter().enumerate() {
                    rows.push(Row::Candidate(ci, ki));
                }
            }
        }
        rows.push(Row::Blank);
        rows.push(Row::Heading(format!(
            "mode {} · {} · m toggles the mode · n toggles dry run",
            match self.mode {
                Mode::Trash => "Trash (recoverable until emptied)",
                Mode::Permanent => "PERMANENT (cannot be undone)",
            },
            if self.dry_run {
                "dry run: nothing is removed"
            } else {
                "live"
            }
        )));
        self.rows = rows;
        let target = keep.and_then(|p| {
            self.rows.iter().position(
                |r| matches!(r, Row::Candidate(c, i) if self.cats[*c].candidates[*i].path == p),
            )
        });
        self.cursor = target
            .unwrap_or(self.cursor)
            .min(self.rows.len().saturating_sub(1));
        if !matches!(
            self.rows.get(self.cursor),
            Some(Row::Category(_) | Row::Candidate(..))
        ) {
            self.cursor = self
                .rows
                .iter()
                .position(|r| matches!(r, Row::Category(_) | Row::Candidate(..)))
                .unwrap_or(0);
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
            if matches!(self.rows[c as usize], Row::Category(_) | Row::Candidate(..)) {
                self.cursor = c as usize;
                self.detail_scroll.jump_start();
                self.scroll.ensure_visible(self.cursor);
                return;
            }
        }
    }

    /// Checked candidates as deletion items, with their guards.
    pub fn selected_items(&self) -> Vec<DeleteItem> {
        let mut v = vec![];
        for c in &self.cats {
            for k in &c.candidates {
                if self.checked.contains(&k.path) && k.eligibility.selectable() {
                    v.push(DeleteItem {
                        path: k.path.clone(),
                        category: self
                            .special
                            .as_ref()
                            .map(|s| s.id.clone())
                            .unwrap_or(c.category.id.into()),
                        estimate: k.bytes,
                        guard: c.category.guard_process,
                    });
                }
            }
        }
        v
    }

    pub fn selected_bytes(&self) -> u64 {
        self.selected_items().iter().map(|i| i.estimate).sum()
    }

    fn build_plan(&self, w: &World) -> DeletePlan {
        let root = match &self.special {
            Some(_) => w.location.cwd.clone(),
            None => w.location.home.clone(),
        };
        DeletePlan::new(
            &w.host.name,
            &root,
            self.selected_items(),
            self.mode,
            self.dry_run,
        )
    }

    fn toggle_candidate(&mut self, ci: usize, ki: usize, cx: &mut Cx) {
        let k = &self.cats[ci].candidates[ki];
        match &k.eligibility {
            Eligibility::Ineligible(why) => {
                cx.status(format!(
                    "Cannot select {} · {why}",
                    k.path.rsplit('/').next().unwrap_or("")
                ));
            }
            _ => {
                if !self.checked.remove(&k.path) {
                    self.checked.insert(k.path.clone());
                }
                cx.status(format!(
                    "{} selected · {}",
                    plural(self.checked.len(), "item", "items"),
                    human(self.selected_bytes())
                ));
            }
        }
    }

    fn toggle_category(&mut self, ci: usize, cx: &mut Cx) {
        let eligible: Vec<String> = self.cats[ci]
            .candidates
            .iter()
            .filter(|k| k.eligibility.selectable())
            .map(|k| k.path.clone())
            .collect();
        if eligible.is_empty() {
            cx.status("No eligible candidate in this category");
            return;
        }
        let all = eligible.iter().all(|p| self.checked.contains(p));
        for p in eligible {
            if all {
                self.checked.remove(&p);
            } else {
                self.checked.insert(p);
            }
        }
        let skipped = self.cats[ci]
            .candidates
            .iter()
            .filter(|k| !k.eligibility.selectable())
            .count();
        cx.status(format!(
            "{} · {} selected · {}{}",
            self.cats[ci].category.label,
            plural(self.checked.len(), "item", "items"),
            human(self.selected_bytes()),
            if skipped > 0 {
                format!(" · {skipped} ineligible stay unselected")
            } else {
                String::new()
            }
        ));
    }

    fn detail(&self, w: &World) -> (String, String, Vec<Prop>) {
        match self.rows.get(self.cursor) {
            Some(Row::Category(ci)) => {
                let c = &self.cats[*ci];
                let cat = c.category;
                let (label, note) = match &self.special {
                    Some(s) => (s.label.clone(), s.note.clone()),
                    None => (cat.label.to_owned(), cat.note.to_owned()),
                };
                let mut v = vec![
                    Prop::new("Safety", cat.safety.label()).tone(match cat.safety {
                        cleanup::Safety::ReviewFirst => Tone::Warning,
                        _ => Tone::Normal,
                    }),
                    Prop::new(
                        "Age rule",
                        if cat.min_age_days > 0 {
                            format!(
                                "older than {} days · unknown or future age counts as recent",
                                cat.min_age_days
                            )
                        } else {
                            "any age".into()
                        },
                    )
                    .wrap(),
                    Prop::new(
                        "Platform",
                        if cat.macos_only {
                            "macOS only"
                        } else {
                            "macOS and Linux"
                        },
                    ),
                    Prop::new(
                        "Roots",
                        cat.roots
                            .iter()
                            .map(|r| format!("~/{r}"))
                            .collect::<Vec<_>>()
                            .join(" · "),
                    )
                    .wrap(),
                    Prop::new("Why", note).wrap(),
                    Prop::new(
                        "Size",
                        format!(
                            "{}{}",
                            human(c.bytes()),
                            if c.candidates.iter().any(|k| k.partial) {
                                " · partial (unreadable entries)"
                            } else {
                                ""
                            }
                        ),
                    ),
                ];
                if !c.unreadable.is_empty() {
                    v.push(
                        Prop::new(
                            "Unreadable",
                            format!(
                                "{} skipped · the list is a lower bound: {}",
                                plural(c.unreadable.len(), "folder", "folders"),
                                c.unreadable
                                    .iter()
                                    .map(|p| w.location.short(p))
                                    .collect::<Vec<_>>()
                                    .join(" · ")
                            ),
                        )
                        .tone(Tone::Warning)
                        .wrap(),
                    );
                }
                if let Some(g) = cat.guard_process {
                    v.push(
                        Prop::new(
                            "Guard",
                            match &c.process {
                                ProcessObservation::Running => {
                                    format!("{g} is running · candidates are skipped")
                                }
                                ProcessObservation::NotRunning => format!("{g} is not running"),
                                ProcessObservation::Unknown(why) => {
                                    format!("{g} state unknown · {why} · treated as blocked")
                                }
                            },
                        )
                        .tone(match c.process {
                            ProcessObservation::NotRunning => Tone::Normal,
                            _ => Tone::Warning,
                        })
                        .wrap(),
                    );
                }
                if let Some(s) = &self.special
                    && let Some(p) = &s.prerequisite
                {
                    v.push(Prop::new("Prerequisite", format!("{p} runs first · a failed or unknown stop prevents the cleanup · dry run never stops the daemon")).tone(Tone::Warning).wrap());
                }
                v.push(Prop::new(
                    "Selection",
                    format!(
                        "{} of {} candidates eligible · {} checked",
                        c.candidates
                            .iter()
                            .filter(|k| k.eligibility.selectable())
                            .count(),
                        c.candidates.len(),
                        c.candidates
                            .iter()
                            .filter(|k| self.checked.contains(&k.path))
                            .count()
                    ),
                ));
                (label, "category".into(), v)
            }
            Some(Row::Candidate(ci, ki)) => {
                let c = &self.cats[*ci];
                let k = &c.candidates[*ki];
                let node = w.fs.get(&k.path);
                let mut v = vec![
                    Prop::new("Path", w.location.short(&k.path)).wrap(),
                    Prop::new("Kind", node.map(|n| n.kind_word()).unwrap_or("missing")),
                    Prop::new(
                        "Size",
                        format!(
                            "{}{}",
                            human(k.bytes),
                            if k.partial { " · lower bound" } else { "" }
                        ),
                    ),
                    Prop::new(
                        "Age",
                        match k.age_days {
                            Some(0) => "modified today".into(),
                            Some(d) => format!("{d} days"),
                            None => "unknown · treated as recent".into(),
                        },
                    )
                    .tone(if k.age_days.is_none() {
                        Tone::Warning
                    } else {
                        Tone::Normal
                    }),
                    Prop::new(
                        "Category",
                        format!("{} · {}", c.category.label, c.category.safety.label()),
                    ),
                    Prop::new(
                        "Eligibility",
                        match &k.eligibility {
                            Eligibility::Preselected => "eligible · selected by default".into(),
                            Eligibility::Selectable => {
                                "eligible · review first, never preselected".into()
                            }
                            Eligibility::Ineligible(why) => format!("ineligible · {why}"),
                        },
                    )
                    .tone(match k.eligibility {
                        Eligibility::Ineligible(_) => Tone::Warning,
                        _ => Tone::Normal,
                    })
                    .wrap(),
                ];
                match cleanup::validate(&k.path, &w.location.home, w.host.os, &w.fs) {
                    Ok(canon) if canon != k.path => v.push(
                        Prop::new("Canonical", w.location.short(&canon))
                            .tone(Tone::Muted)
                            .wrap(),
                    ),
                    Ok(_) => {}
                    Err(cleanup::Deny(why)) => {
                        v.push(Prop::new("Denied", why).tone(Tone::Error).wrap())
                    }
                }
                v.push(
                    Prop::new(
                        "Method",
                        match (self.mode, self.dry_run) {
                            (_, true) => "dry run · would be removed · nothing changes",
                            (Mode::Trash, false) => {
                                "move to Trash · recoverable until the Trash is emptied"
                            }
                            (Mode::Permanent, false) => "permanent · cannot be undone",
                        },
                    )
                    .tone(if self.mode == Mode::Permanent && !self.dry_run {
                        Tone::Error
                    } else {
                        Tone::Normal
                    })
                    .wrap(),
                );
                (
                    k.path.rsplit('/').next().unwrap_or("").to_owned(),
                    "candidate".into(),
                    v,
                )
            }
            _ => ("Cleanup".into(), String::new(), vec![]),
        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(LIST);
        self.scroll.set_content(self.rows.len());
        self.scroll.set_viewport(area.height as usize);
        ctx.control(LIST, area, false);
        ctx.scrollable(LIST, area);
        let has_sb = self.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let row = Rect::new(area.x, y, row_w, 1);
            match &self.rows[i] {
                Row::Blank => {}
                Row::Heading(h) => heading(buf, area.x + 3, y, row_w.saturating_sub(3), h, t, bg),
                Row::Note(n) => buf.set_string(
                    area.x + 5,
                    y,
                    truncate(n, row_w.saturating_sub(5) as usize),
                    t.muted(),
                ),
                Row::Category(ci) => {
                    let c = &self.cats[*ci];
                    let rid = LIST.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.selected = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(
                        row.x,
                        y,
                        t.gutter_symbol(s),
                        t.gutter(s, st.bg.unwrap_or(bg), false),
                    );
                    let plain = st.remove_modifier(Modifier::BOLD);
                    let eligible = c
                        .candidates
                        .iter()
                        .filter(|k| k.eligibility.selectable())
                        .count();
                    let checked = c
                        .candidates
                        .iter()
                        .filter(|k| self.checked.contains(&k.path))
                        .count();
                    let mark = if eligible == 0 {
                        "[–]"
                    } else if checked == eligible {
                        "[✓]"
                    } else if checked > 0 {
                        "[~]"
                    } else {
                        "[ ]"
                    };
                    buf.set_string(
                        row.x + 1,
                        y,
                        mark,
                        plain.fg(if checked > 0 { t.accent } else { t.text_muted }),
                    );
                    let fold = if self.open.contains(ci) { "▾" } else { "▸" };
                    buf.set_string(row.x + 5, y, fold, plain.fg(t.text_secondary));
                    let label = match &self.special {
                        Some(sp) => sp.label.clone(),
                        None => c.category.label.to_owned(),
                    };
                    let label_w = (row_w * 34 / 100).clamp(16, 40);
                    buf.set_string(row.x + 7, y, fit(&label, label_w as usize), st);
                    let mut x = row.x + 7 + label_w + 2;
                    let size = format!("{:>9}", human(c.bytes()));
                    if x + 10 <= row.right() {
                        buf.set_string(x, y, &size, plain.fg(t.text_secondary));
                    }
                    x += 11;
                    let meta = format!(
                        "{} · {} · {}",
                        c.category.safety.label(),
                        plural(c.candidates.len(), "candidate", "candidates"),
                        match &c.process {
                            ProcessObservation::Running =>
                                format!("{} running", c.category.guard_process.unwrap_or("app")),
                            ProcessObservation::Unknown(_) => "process state unknown".into(),
                            ProcessObservation::NotRunning =>
                                if c.category.min_age_days > 0 {
                                    format!("≥ {} d", c.category.min_age_days)
                                } else {
                                    "any age".into()
                                },
                        }
                    );
                    let avail = row.right().saturating_sub(x + 1) as usize;
                    if avail >= 8 {
                        buf.set_string(
                            x,
                            y,
                            crate::screens::truncate_sep(&meta, avail),
                            plain.fg(match c.process {
                                ProcessObservation::NotRunning => t.text_muted,
                                _ => t.warning,
                            }),
                        );
                    }
                    ctx.clickable(rid, row);
                }
                Row::Candidate(ci, ki) => {
                    let c = &self.cats[*ci];
                    let k = &c.candidates[*ki];
                    let rid = LIST.child(i);
                    let mut s = ctx.state(rid);
                    s.focused = focused && i == self.cursor;
                    s.selected = i == self.cursor;
                    let st = t.row(s, bg);
                    fill(buf, row, st);
                    buf.set_string(
                        row.x,
                        y,
                        t.gutter_symbol(s),
                        t.gutter(s, st.bg.unwrap_or(bg), false),
                    );
                    let plain = st.remove_modifier(Modifier::BOLD);
                    let ineligible = !k.eligibility.selectable();
                    let quiet = if s.selected {
                        t.text_muted
                    } else {
                        t.text_faint
                    };
                    let checked = self.checked.contains(&k.path);
                    let (mark, ms) = if ineligible {
                        ("[ ]", plain.fg(quiet))
                    } else if checked {
                        ("[✓]", plain.fg(t.accent))
                    } else {
                        ("[ ]", plain.fg(t.text_muted))
                    };
                    buf.set_string(row.x + 3, y, mark, ms);
                    let label = w.location.short(&k.path);
                    let label_w = (row_w * 40 / 100).clamp(16, 48);
                    buf.set_string(
                        row.x + 7,
                        y,
                        fit(&label, label_w as usize),
                        if ineligible { st.fg(quiet) } else { st },
                    );
                    let mut x = row.x + 7 + label_w + 2;
                    if x + 10 <= row.right() {
                        buf.set_string(
                            x,
                            y,
                            format!("{:>9}", human(k.bytes)),
                            plain.fg(t.text_secondary),
                        );
                    }
                    x += 11;
                    let age = match k.age_days {
                        Some(0) => "today".to_owned(),
                        Some(d) => format!("{d} d"),
                        None => "age unknown".into(),
                    };
                    if x + 12 <= row.right() {
                        buf.set_string(
                            x,
                            y,
                            fit(&age, 12),
                            plain.fg(if k.age_days.is_none() {
                                t.warning
                            } else {
                                t.text_muted
                            }),
                        );
                    }
                    x += 13;
                    let tail = match &k.eligibility {
                        Eligibility::Ineligible(why) => why.clone(),
                        Eligibility::Selectable => "review first".into(),
                        Eligibility::Preselected => String::new(),
                    };
                    let avail = row.right().saturating_sub(x + 1) as usize;
                    if avail >= 6 && !tail.is_empty() {
                        buf.set_string(
                            x,
                            y,
                            truncate(&tail, avail),
                            plain.fg(if ineligible { quiet } else { t.text_faint }),
                        );
                    }
                    ctx.clickable(rid, row);
                }
            }
        }
        if has_sb {
            junie_tui::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    area.x,
                    area.y,
                    (area.right() - 1).saturating_sub(area.x),
                    area.height,
                ),
                &self.scroll,
            );
            scrollbar::render_vertical(
                Rect::new(area.right() - 1, area.y, 1, area.height),
                buf,
                ctx,
                LIST,
                &self.scroll,
                focused,
            );
        }
    }

    fn render_detail(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let focused = ctx.interaction.focused(DETAIL);
        let (title, meta, props) = self.detail(w);
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
            junie_tui::ui::fade::scroll_edges(
                buf,
                ctx,
                Rect::new(
                    inner.x,
                    inner.y,
                    (inner.right() - 1).saturating_sub(inner.x),
                    inner.height,
                ),
                &self.detail_scroll,
            );
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

    fn locate(&self, id: WidgetId) -> Option<usize> {
        self.scroll.visible_range().find(|&i| LIST.child(i) == id)
    }
}

impl Screen for CleanupPage {
    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if cx.focus.is(DETAIL) {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') if key.plain() => {
                    self.detail_scroll.scroll_by(-1);
                    return Outcome::Changed;
                }
                KeyCode::Down | KeyCode::Char('j') if key.plain() => {
                    self.detail_scroll.scroll_by(1);
                    return Outcome::Changed;
                }
                KeyCode::Esc => {
                    self.drawer = false;
                    cx.focus.focus(LIST);
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        if matches!(key.code, KeyCode::Char(_)) && (key.ctrl() || key.alt()) {
            return Outcome::Ignored;
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
                if !matches!(
                    self.rows.first(),
                    Some(Row::Category(_) | Row::Candidate(..))
                ) {
                    self.step(1);
                }
                self.scroll.jump_start();
                Outcome::Changed
            }
            KeyCode::End | KeyCode::Char('G') => {
                self.cursor = self.rows.len();
                self.step(-1);
                Outcome::Changed
            }
            KeyCode::Char(' ') => {
                match self.rows.get(self.cursor).cloned() {
                    Some(Row::Category(ci)) => self.toggle_category(ci, cx),
                    Some(Row::Candidate(ci, ki)) => self.toggle_candidate(ci, ki, cx),
                    _ => cx.status("Move onto a category or candidate first"),
                }
                self.rebuild(w);
                Outcome::Changed
            }
            KeyCode::Enter | KeyCode::Right | KeyCode::Left => {
                match self.rows.get(self.cursor).cloned() {
                    Some(Row::Category(ci)) => {
                        if key.code == KeyCode::Left
                            || (key.code == KeyCode::Enter && self.open.contains(&ci))
                        {
                            self.open.remove(&ci);
                        } else {
                            self.open.insert(ci);
                        }
                        self.rebuild(w);
                    }
                    Some(Row::Candidate(ci, ki)) => {
                        if key.code == KeyCode::Enter {
                            self.toggle_candidate(ci, ki, cx);
                            self.rebuild(w);
                        } else if key.code == KeyCode::Left {
                            self.open.remove(&ci);
                            self.rebuild(w);
                            self.cursor = self
                                .rows
                                .iter()
                                .position(|r| matches!(r, Row::Category(c) if *c == ci))
                                .unwrap_or(0);
                        }
                    }
                    _ => {}
                }
                Outcome::Changed
            }
            KeyCode::Char('a') => {
                let eligible: Vec<String> = self
                    .cats
                    .iter()
                    .flat_map(|c| c.candidates.iter())
                    .filter(|k| k.eligibility.selectable())
                    .map(|k| k.path.clone())
                    .collect();
                let all = eligible.iter().all(|p| self.checked.contains(p));
                for p in eligible {
                    if all {
                        self.checked.remove(&p);
                    } else {
                        self.checked.insert(p);
                    }
                }
                cx.status(if all {
                    "Everything unselected".to_owned()
                } else {
                    format!(
                        "Every eligible candidate selected · {}",
                        human(self.selected_bytes())
                    )
                });
                self.rebuild(w);
                Outcome::Changed
            }
            KeyCode::Char('m') => {
                self.mode = match self.mode {
                    Mode::Trash => Mode::Permanent,
                    Mode::Permanent => Mode::Trash,
                };
                self.reviewed = None;
                cx.status(match self.mode {
                    Mode::Trash => "Mode: Trash · recoverable until the Trash is emptied",
                    Mode::Permanent => {
                        "Mode: PERMANENT · cannot be undone · any earlier review is void"
                    }
                });
                self.rebuild(w);
                Outcome::Changed
            }
            KeyCode::Char('n') => {
                self.dry_run = !self.dry_run;
                self.reviewed = None;
                cx.status(if self.dry_run { "Dry run: paths are validated and sized, the log records would-remove, nothing is deleted and no daemon is stopped" } else { "Live run" });
                self.rebuild(w);
                Outcome::Changed
            }
            KeyCode::Char('d') | KeyCode::Backspace => {
                let plan = self.build_plan(w);
                if plan.items.is_empty() {
                    cx.status("Select at least one eligible candidate first · Space selects");
                    return Outcome::Changed;
                }
                self.reviewed = Some(plan.revision);
                cx.go(Go::Push(Page::CleanupGate { plan }));
                Outcome::Changed
            }
            KeyCode::Char('p') => {
                self.drawer = !self.drawer;
                cx.focus.focus(if self.drawer { DETAIL } else { LIST });
                Outcome::Changed
            }
            KeyCode::Char('y') => {
                if let Some(Row::Candidate(ci, ki)) = self.rows.get(self.cursor) {
                    cx.copy(self.cats[*ci].candidates[*ki].path.clone());
                }
                Outcome::Changed
            }
            KeyCode::Esc => {
                if self.filter.is_none() && !self.open.is_empty() {
                    self.open.clear();
                    self.rebuild(w);
                    return Outcome::Changed;
                }
                cx.go(Go::Pop);
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
            cx.focus.focus(LIST);
            match self.rows.get(i).cloned() {
                Some(Row::Category(ci)) => {
                    if !self.open.remove(&ci) {
                        self.open.insert(ci);
                    }
                    self.rebuild(w);
                }
                Some(Row::Candidate(ci, ki)) => {
                    self.toggle_candidate(ci, ki, cx);
                    self.rebuild(w);
                }
                _ => {}
            }
            return Outcome::Changed;
        }
        if id == LIST {
            cx.focus.focus(LIST);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == LIST {
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

    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.rebuild(w);
        cx.focus.focus(LIST);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        if self.rows.is_empty() {
            self.rebuild(w);
        }
        let t = ctx.theme;
        let title = match &self.special {
            Some(s) => format!("Cleanup · {}", s.label),
            None => match &self.filter {
                Some(f) => format!(
                    "Cleanup · {}",
                    cleanup::category(f).map(|c| c.label).unwrap_or(f)
                ),
                None => "Cleanup review".into(),
            },
        };
        buf.set_string(area.x + 1, area.y, &title, t.title());
        let total: u64 = self.cats.iter().map(InsightCategory::bytes).sum();
        let line = format!(
            "{} · {} · {} selected ({}) · {}{}",
            plural(self.cats.len(), "category", "categories"),
            human(total),
            self.selected_items().len(),
            human(self.selected_bytes()),
            match self.mode {
                Mode::Trash => "Trash",
                Mode::Permanent => "PERMANENT",
            },
            if self.dry_run { " · dry run" } else { "" }
        );
        buf.set_string(
            area.x + 1,
            area.y + 1,
            truncate(&line, area.width.saturating_sub(2) as usize),
            if self.mode == Mode::Permanent && !self.dry_run {
                Style::new().fg(t.error)
            } else {
                t.muted()
            },
        );
        let body = Rect::new(
            area.x,
            area.y + 3,
            area.width,
            area.height.saturating_sub(3),
        );
        let split = area.width >= crate::screens::finder::SPLIT_MIN;
        if split {
            let dw = (area.width * 38 / 100).clamp(34, 50);
            let list = Rect::new(
                body.x,
                body.y,
                body.width.saturating_sub(dw + 2),
                body.height,
            );
            let detail = Rect::new(list.right() + 2, body.y, dw, body.height);
            self.render_list(list, buf, ctx, w);
            self.render_detail(detail, buf, ctx, w);
        } else if (self.drawer || ctx.interaction.focused(DETAIL)) && !ctx.interaction.focused(LIST)
        {
            self.drawer = true;
            ctx.control(LIST, Rect::ZERO, false);
            self.render_detail(body, buf, ctx, w);
        } else {
            // focus on the list gives the body back to it
            self.drawer = false;
            self.render_list(body, buf, ctx, w);
            ctx.control(DETAIL, Rect::ZERO, false);
        }
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus == Some(DETAIL) {
            return vec![hint("↑↓", "Scroll"), hint("Esc", "List")];
        }
        let mut v = vec![
            hint("↑↓", "Move"),
            hint("Space", "Select"),
            hint("Enter", "Open"),
        ];
        v.push(hint("a", "All"));
        v.push(hint(
            "m",
            match self.mode {
                Mode::Trash => "Permanent",
                Mode::Permanent => "Trash",
            },
        ));
        v.push(hint("n", if self.dry_run { "Live" } else { "Dry run" }));
        v.push(hint("d", "Review deletion"));
        v.push(hint("p", "Facts"));
        v.push(hint("Esc", "Back"));
        v
    }

    fn crumb(&self, _w: &World) -> String {
        match &self.filter {
            Some(f) => format!(
                "Cleanup › {}",
                cleanup::category(f).map(|c| c.label).unwrap_or(f)
            ),
            None => "Cleanup".into(),
        }
    }

    fn status(&self, _w: &World) -> StatusBits {
        StatusBits {
            center: Some(
                StatusItem::new(
                    format!(
                        "{} selected · {}{}",
                        self.selected_items().len(),
                        human(self.selected_bytes()),
                        if self.dry_run { " · dry run" } else { "" }
                    ),
                    Tone::Secondary,
                )
                .priority(6),
            ),
            right: vec![],
        }
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(LIST)
    }
}

/// Gate 1 facts for a deletion plan: resolved before review, revalidated
/// at commit.
pub fn plan_facts(plan: &DeletePlan, w: &World) -> Vec<Prop> {
    let home = w.location.home.clone();
    let denied = plan
        .items
        .iter()
        .filter(|i| {
            cleanup::validate(&i.path, &home, w.host.os, &w.fs).is_err()
                || cleanup::protected_descendant(&i.path, &home, &w.fs).is_some()
        })
        .count();
    let guarded: Vec<&str> = plan
        .items
        .iter()
        .filter_map(|i| i.guard)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let mut v = vec![
        Prop::new("Action", format!(
            "{} {} under {}",
            if plan.dry_run { "dry run over" } else if plan.mode == Mode::Permanent { "permanently delete" } else { "move to Trash" },
            plural(plan.items.len(), "item", "items"),
            w.location.short(&plan.root)
        )).wrap(),
        Prop::new("Host", format!("{} · {}", w.host.name, w.host.role.label())),
        Prop::new("Mode", match (plan.mode, plan.dry_run) {
            (_, true) => "dry run · validate and size · log would-remove · touch nothing".to_owned(),
            (Mode::Trash, false) => format!("Trash · {} · space returns after the Trash is emptied", match &w.platform.trash {
                crate::domain::stack::TrashBackend::MacNative => "native macOS Trash",
                crate::domain::stack::TrashBackend::FreeDesktop => "FreeDesktop trash",
                crate::domain::stack::TrashBackend::Unavailable(_) => "no Trash backend · every item will fail, never fall back",
            }),
            (Mode::Permanent, false) => "PERMANENT · unlink and remove_dir_all · cannot be undone".into(),
        }).tone(if plan.mode == Mode::Permanent && !plan.dry_run { Tone::Error } else { Tone::Normal }).wrap(),
        Prop::new("Estimate", format!("{} allocated · APFS clones may overcount · purgeable space excluded · not a free-space guarantee", human(plan.estimate()))).wrap(),
        Prop::new("Denied", if denied == 0 { "none · every path passed the lexical, protected-root, user-data and symlink rules".to_owned() } else { format!("{denied} would be skipped at commit by the safety rules") }).tone(if denied > 0 { Tone::Warning } else { Tone::Muted }).wrap(),
    ];
    if !guarded.is_empty() {
        v.push(
            Prop::new(
                "Guard",
                format!(
                    "{} · observed once at commit · a running or unknown process skips its items",
                    guarded.join(", ")
                ),
            )
            .tone(Tone::Warning)
            .wrap(),
        );
    }
    if plan.items.iter().any(|i| i.category == "gradle.clean-all") {
        v.push(
            Prop::new(
                "Prerequisite",
                if plan.dry_run {
                    "gradle --stop is not run on a dry run"
                } else {
                    "gradle --stop first · a failed or unknown stop cancels the cleanup"
                },
            )
            .tone(Tone::Warning)
            .wrap(),
        );
    }
    v.push(
        Prop::new(
            "Revision",
            format!(
                "{:016x} · any change to paths, mode or dry run voids this review",
                plan.revision
            ),
        )
        .tone(Tone::Muted)
        .wrap(),
    );
    v.push(Prop::new("Threat model", "an unprivileged user · ancestors are re-resolved at commit · a concurrent rename of the final path cannot be excluded").tone(Tone::Muted).wrap());
    let shown: Vec<String> = plan
        .items
        .iter()
        .take(10)
        .map(|i| w.location.short(&i.path))
        .collect();
    v.push(
        Prop::new(
            "Paths",
            format!(
                "{}{}",
                shown.join(" · "),
                if plan.items.len() > 10 {
                    format!(" · … {} more", plan.items.len() - 10)
                } else {
                    String::new()
                }
            ),
        )
        .wrap(),
    );
    v
}

/// Commit an authorized plan: prerequisites, then the execution starts as a
/// world-owned job that `step` advances one item per tick. Returns the index
/// of the report that grows while it runs.
pub fn commit(plan: &DeletePlan, w: &mut World) -> Result<usize, String> {
    if plan.fingerprint() != plan.revision {
        return Err("the reviewed plan changed · confirm again".into());
    }
    if w.cleanup_job.is_some() {
        return Err("a cleanup is still running · wait for its report".into());
    }
    if !plan.dry_run && plan.items.iter().any(|i| i.category == "gradle.clean-all") {
        let stop = crate::domain::exec::Command::argv(
            "gradle",
            &["--stop"],
            &w.location.cwd,
            &w.host.name,
        );
        let script = crate::domain::outcomes::for_command(w, &stop)
            .unwrap_or_else(|| crate::domain::activity::Script::unmodeled("gradle --stop"));
        if script.exit != 0 || script.spawn_failure.is_some() {
            let reason = script
                .lines
                .iter()
                .rev()
                .find(|l| l.tone == crate::domain::activity::LineTone::Error)
                .map(|l| l.text.clone())
                .or(script.spawn_failure.clone())
                .unwrap_or("unknown".into());
            return Err(format!(
                "gradle --stop failed ({reason}) · daemon state unknown · nothing was removed"
            ));
        }
        w.apply_effect(&crate::domain::effect::Effect::GradleStop);
    }
    let procs: Vec<(String, ProcessObservation)> = plan
        .items
        .iter()
        .filter_map(|i| i.guard)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .map(|g| (g.to_owned(), observe_process(w, g)))
        .collect();
    let exec = cleanup::Execution::start(plan, plan.revision, &w.ops_log.path)?;
    w.reports.push(exec.report().clone());
    let report_index = w.reports.len() - 1;
    w.cleanup_job = Some(crate::sim::world::CleanupJob {
        exec,
        report_index,
        processes: procs,
    });
    Ok(report_index)
}

/// Advance the running cleanup by one item. Returns the report index on the
/// step that settles it; the partial report is visible in between.
pub fn step(w: &mut World) -> Option<usize> {
    let mut job = w.cleanup_job.take()?;
    let home = w.location.home.clone();
    let os = w.host.os;
    let now = w.now_secs();
    let procs = job.processes.clone();
    let observe = |name: &str| -> ProcessObservation {
        procs
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, o)| o.clone())
            .unwrap_or(ProcessObservation::NotRunning)
    };
    let mut fs = std::mem::take(&mut w.fs);
    let mut log = std::mem::take(&mut w.ops_log);
    job.exec.step(
        &mut fs,
        &mut log,
        &cleanup::ExecContext {
            home: &home,
            os,
            now_secs: now,
            processes: &observe,
        },
    );
    w.fs = fs;
    let idx = job.report_index;
    if job.exec.finished() {
        let report = job.exec.finish(&mut log);
        w.ops_log = log;
        settle(w, idx, report);
        Some(idx)
    } else {
        w.ops_log = log;
        w.reports[idx] = job.exec.report().clone();
        w.cleanup_job = Some(job);
        None
    }
}

/// Land a finished cleanup: log persistence, world effects and a rescan.
fn settle(w: &mut World, idx: usize, report: cleanup::Report) {
    w.persisted.ops_log = w.ops_log.lines.clone();
    // legacy candidate rows follow the filesystem
    let removed: Vec<String> = report
        .items
        .iter()
        .filter(|i| {
            matches!(
                i.outcome,
                cleanup::Outcome::Removed | cleanup::Outcome::Trashed
            )
        })
        .map(|i| i.path.clone())
        .collect();
    // the operation log is the record of a gated cleanup; the overview's
    // candidate list only drops what left the tree
    for p in &removed {
        w.disk
            .candidates
            .retain(|c| !(c.path == *p || c.path.starts_with(&format!("{p}/"))));
    }
    if report.freed_now > 0
        && let Some(fs) = w.disk.filesystems.first_mut()
    {
        fs.used_gb = fs
            .used_gb
            .saturating_sub((report.freed_now / (1024 * 1024 * 1024)) as u32);
    }
    w.reports[idx] = report;
    if let Some(root) = w.scan.as_ref().map(|s| s.root.clone()) {
        w.start_scan(&root);
    }
}
