//! Deciding pages: Gate 1 (the resolved facts and the full command
//! sequence before a destructive action), the trust review for an untrusted
//! definition, and structured arguments.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::button::{Button, row_layout_right};
use junie_tui::widgets::input::{InputEvent, TextInput};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::props::{self, Prop};
use junie_tui::widgets::scrollbar;
use junie_tui::widgets::select::Select;
use junie_tui::widgets::statusbar::StatusItem;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::domain::action::{ArgSpec, Item, Risk};
use crate::screens::finder::item_facts;
use crate::screens::{Cx, GateTarget, Go, Screen, StatusBits, heading, plural, risk_tone};
use crate::sim::world::World;

pub const SEQUENCE: WidgetId = WidgetId::of("gate.sequence");
pub const GATE_CANCEL: WidgetId = WidgetId::of("gate.cancel");
pub const GATE_CONTINUE: WidgetId = WidgetId::of("gate.continue");

// ------------------------------------------------------------------ gate 1

pub struct GatePage {
    pub target: GateTarget,
    title: String,
    facts: Vec<Prop>,
    sequence: Vec<String>,
    scroll: ScrollState,
    cancel: Button,
    cont: Button,
}

impl GatePage {
    pub fn new(target: GateTarget, w: &World) -> Self {
        let mut page = Self {
            target,
            title: String::new(),
            facts: vec![],
            sequence: vec![],
            scroll: ScrollState::default(),
            cancel: Button::secondary(GATE_CANCEL, "Cancel"),
            cont: Button::danger(GATE_CONTINUE, "Continue…"),
        };
        page.resolve(w);
        page
    }

    /// Re-resolve every fact from the live world (CONCEPT §10: the review
    /// shows the plan as it is now).
    fn resolve(&mut self, w: &World) {
        let host_tone = if w.host.role.sensitive() {
            Tone::Warning
        } else {
            Tone::Normal
        };
        match &self.target {
            GateTarget::Plan(id) => {
                let Some(p) = w.plan(id) else {
                    return;
                };
                self.title = format!("Review · {}", p.title);
                let mut f = vec![
                    Prop::new("Action", p.intent.clone()).wrap(),
                    Prop::new(
                        "Host",
                        format!(
                            "{} · {}{}",
                            p.host,
                            w.host.role.label(),
                            if w.host.remote { " · over SSH" } else { "" }
                        ),
                    )
                    .tone(host_tone),
                    Prop::new("In", w.location.cwd_short()),
                    Prop::new(
                        "Steps",
                        format!(
                            "{} included · {} excluded · {} parallel · ~{}",
                            p.included_count(),
                            p.steps.len() - p.included_count(),
                            plural(p.parallel_branches(), "branch", "branches"),
                            crate::screens::ticks_label(p.estimate_ticks())
                        ),
                    ),
                ];
                for (k, v) in &p.facts {
                    f.push(Prop::new(k, v.clone()).wrap());
                }
                let priv_: Vec<String> = p
                    .steps
                    .iter()
                    .filter(|s| s.included && !s.privilege.is_empty())
                    .map(|s| s.privilege.clone())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect();
                f.push(
                    Prop::new(
                        "Privilege",
                        if priv_.is_empty() {
                            "none".into()
                        } else {
                            priv_.join(", ")
                        },
                    )
                    .tone(if priv_.is_empty() {
                        Tone::Muted
                    } else {
                        Tone::Warning
                    }),
                );
                let notes: Vec<String> = p
                    .steps
                    .iter()
                    .filter(|s| s.included && !s.note.is_empty())
                    .map(|s| s.note.clone())
                    .collect();
                if !notes.is_empty() {
                    f.push(
                        Prop::new("Uncertainty", notes.join(" · "))
                            .tone(Tone::Warning)
                            .wrap(),
                    );
                }
                f.push(
                    Prop::new(
                        "Provenance",
                        format!(
                            "resolved by holla from live state · {}",
                            w.clock.ago(w.now_secs() - 2)
                        ),
                    )
                    .tone(Tone::Muted),
                );
                if p.drift_consumed
                    && let Some(d) = &p.drift
                {
                    f.push(
                        Prop::new(
                            "Changed",
                            format!("since the last review: {d} · confirm again"),
                        )
                        .tone(Tone::Error)
                        .wrap(),
                    );
                }
                f.push(
                    Prop::new(
                        "Gate",
                        if p.phrase.is_some() {
                            "second gate types the target-bound phrase"
                        } else {
                            "one confirmation"
                        },
                    )
                    .tone(Tone::Muted),
                );
                self.facts = f;
                self.sequence = p
                    .steps
                    .iter()
                    .filter(|s| s.included)
                    .flat_map(|s| {
                        s.commands
                            .iter()
                            .map(move |c| format!("{}  # {:02} {}", c, 0, s.label))
                    })
                    .collect();
                // number the sequence by step
                let mut seq = vec![];
                for (i, s) in p.steps.iter().enumerate() {
                    if !s.included {
                        continue;
                    }
                    for (k, c) in s.commands.iter().enumerate() {
                        seq.push(if k == 0 {
                            format!("{:02}  {c}", i + 1)
                        } else {
                            format!("    {c}")
                        });
                    }
                }
                self.sequence = seq;
            }
            GateTarget::Cleanup(plan) => {
                self.title = format!(
                    "Review · {} {}",
                    if plan.dry_run {
                        "dry run"
                    } else {
                        plan.mode.label()
                    },
                    plural(plan.items.len(), "item", "items")
                );
                let mut f = crate::screens::cleanup::plan_facts(plan, w);
                f.push(
                    Prop::new(
                        "Recoverable",
                        match (plan.mode, plan.dry_run) {
                            (_, true) => "nothing changes".to_owned(),
                            (crate::domain::cleanup::Mode::Trash, _) => {
                                "yes · until the Trash is emptied".to_owned()
                            }
                            (crate::domain::cleanup::Mode::Permanent, _) => {
                                "no · permanent".to_owned()
                            }
                        },
                    )
                    .tone(
                        if plan.mode == crate::domain::cleanup::Mode::Permanent && !plan.dry_run {
                            Tone::Error
                        } else {
                            Tone::Normal
                        },
                    ),
                );
                f.push(
                    Prop::new(
                        "Provenance",
                        "holla cleanup · paths validated at review and again at commit",
                    )
                    .tone(Tone::Muted),
                );
                f.push(
                    Prop::new(
                        "Gate",
                        if plan.dry_run {
                            "one confirmation · nothing is removed"
                        } else {
                            "second gate types the target-bound phrase"
                        },
                    )
                    .tone(Tone::Muted),
                );
                self.facts = f;
                let verb = match (plan.mode, plan.dry_run) {
                    (_, true) => "would-remove",
                    (crate::domain::cleanup::Mode::Trash, _) => "trash",
                    (crate::domain::cleanup::Mode::Permanent, _) => "remove",
                };
                self.sequence = plan
                    .items
                    .iter()
                    .enumerate()
                    .map(|(i, it)| {
                        format!(
                            "{:02}  {verb} {}  # {} · {}",
                            i + 1,
                            crate::domain::exec::quote(&it.path),
                            crate::sim::fs::human(it.estimate),
                            it.category
                        )
                    })
                    .collect();
            }
            GateTarget::Item { item, args } => {
                let Some(it) = w.items().into_iter().find(|i| &i.id == item) else {
                    return;
                };
                self.title = format!("Review · {}", it.label);
                let mut f = item_facts(&it, w, &junie_tui::theme::Theme::junie());
                if !args.is_empty() {
                    f.push(Prop::new(
                        "Arguments",
                        args.iter()
                            .map(|(k, v)| format!("{k} = {v}"))
                            .collect::<Vec<_>>()
                            .join(" · "),
                    ));
                }
                if it.id == "disk.delete_all" {
                    f.push(Prop::new("Includes", "hidden files · nested folders · symlinks are removed, never followed · mounts are skipped").wrap());
                }
                if it.id.starts_with("docker.") {
                    f.push(Prop::new("Includes", "stopped containers and dangling resources · named volumes when volumes are pruned").wrap());
                }
                f.push(
                    Prop::new(
                        "Recoverable",
                        match it.risk {
                            Risk::Destructive => "no · permanent".to_owned(),
                            Risk::Privileged => "service restarts · brief outage".to_owned(),
                            _ => "yes".to_owned(),
                        },
                    )
                    .tone(risk_tone(it.risk)),
                );
                f.push(
                    Prop::new("Provenance", "built-in action · resolved from live state")
                        .tone(Tone::Muted),
                );
                f.push(
                    Prop::new("Gate", "second gate types the target-bound phrase")
                        .tone(Tone::Muted),
                );
                self.facts = f;
                let exec = it.all_exec();
                self.sequence = if exec.is_empty() {
                    it.commands
                        .iter()
                        .enumerate()
                        .map(|(i, c)| format!("{:02}  {c}", i + 1))
                        .collect()
                } else {
                    exec.iter()
                        .enumerate()
                        .map(|(i, c)| format!("{:02}  {}", i + 1, c.display()))
                        .collect()
                };
            }
        }
    }
}

impl Screen for GatePage {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        if cx.focus.is(GATE_CANCEL) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                cx.go(Go::Pop);
                cx.status("Cancelled · nothing was executed");
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cx.focus.is(GATE_CONTINUE) {
            let (o, fired) = self.cont.on_key(key);
            if fired {
                cx.go(Go::Gate1Accepted(self.target.clone()));
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if cx.focus.is(SEQUENCE) => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') if cx.focus.is(SEQUENCE) => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Char('h') if cx.focus.is(GATE_CONTINUE) => {
                cx.focus.focus(GATE_CANCEL);
                Outcome::Changed
            }
            KeyCode::Right | KeyCode::Char('l') if cx.focus.is(GATE_CANCEL) => {
                cx.focus.focus(GATE_CONTINUE);
                Outcome::Changed
            }
            KeyCode::Char('y') => {
                cx.copy(self.sequence.join("\n"));
                Outcome::Changed
            }
            KeyCode::Esc => {
                cx.go(Go::Pop);
                cx.status("Cancelled · nothing was executed");
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, _w: &mut World, cx: &mut Cx) -> Outcome {
        if id == GATE_CANCEL {
            cx.go(Go::Pop);
            cx.status("Cancelled · nothing was executed");
            return Outcome::Changed;
        }
        if id == GATE_CONTINUE {
            cx.go(Go::Gate1Accepted(self.target.clone()));
            return Outcome::Changed;
        }
        if id == SEQUENCE {
            cx.focus.focus(SEQUENCE);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == SEQUENCE {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.resolve(w);
        // cancel is the default outcome: the selecting interaction never counts
        cx.focus.focus(GATE_CANCEL);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        buf.set_string(area.x + 1, area.y, &self.title, t.title());
        let sub = " · gate 1 of 2 · review the resolved facts, then continue deliberately";
        buf.set_string(
            area.x + 1 + width(&self.title) as u16,
            area.y,
            truncate(
                sub,
                area.width.saturating_sub(width(&self.title) as u16 + 2) as usize,
            ),
            t.muted(),
        );
        let facts_h = (self.facts.len() as u16 + 2).min(area.height / 2);
        let facts_area = Rect::new(
            area.x + 1,
            area.y + 2,
            area.width.saturating_sub(2),
            facts_h,
        );
        let used = props::render(facts_area, buf, t, &self.facts, t.canvas);
        let buttons_y = area.bottom().saturating_sub(1);
        let seq_y = facts_area.y + used + 1;
        let seq_area = Rect::new(
            area.x,
            seq_y,
            area.width,
            buttons_y.saturating_sub(seq_y + 1),
        );
        let focused = ctx.interaction.focused(SEQUENCE);
        let title = format!(
            "Sequence · {}",
            plural(self.sequence.len(), "command", "commands")
        );
        let panel = Panel::framed(Some(&title))
            .focused(focused)
            .meta("y copies");
        let inner = panel.render(seq_area, buf, t);
        self.scroll.set_content(self.sequence.len());
        self.scroll.set_viewport(inner.height as usize);
        ctx.control(SEQUENCE, seq_area, false);
        ctx.scrollable(SEQUENCE, inner);
        let has_sb = self.scroll.overflows();
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = inner.y + k as u16;
            let line = &self.sequence[i];
            let (num, rest) = line.split_at(line.len().min(4));
            buf.set_string(inner.x, y, num, t.faint());
            buf.set_string(
                inner.x + 4,
                y,
                truncate(
                    rest,
                    inner.width.saturating_sub(4 + u16::from(has_sb)) as usize,
                ),
                t.secondary(),
            );
        }
        if has_sb {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                SEQUENCE,
                &self.scroll,
                focused,
            );
        }
        let widths = [self.cancel.width(), self.cont.width()];
        let rects = row_layout_right(
            Rect::new(area.x, buttons_y, area.width.saturating_sub(1), 1),
            &widths,
            2,
        );
        self.cancel.render(rects[0], buf, ctx, t.canvas);
        self.cont.render(rects[1], buf, ctx, t.canvas);
        let _ = w;
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus == Some(SEQUENCE) {
            return vec![
                hint("↑↓", "Scroll"),
                hint("y", "Copy sequence"),
                hint("Tab", "Buttons"),
                hint("Esc", "Cancel"),
            ];
        }
        vec![
            hint("← →", "Choose"),
            hint("Enter", "Activate"),
            hint("Tab", "Next"),
            hint("y", "Copy sequence"),
            hint("Esc", "Cancel"),
        ]
    }

    fn crumb(&self, _w: &World) -> String {
        "Review".into()
    }

    fn status(&self, _w: &World) -> StatusBits {
        StatusBits {
            center: Some(StatusItem::new("review · gate 1 of 2", Tone::Secondary).priority(7)),
            right: vec![],
        }
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(GATE_CANCEL)
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.go(Go::Pop);
        Outcome::Changed
    }
}

// ------------------------------------------------------------------ trust

pub const TRUST_CANCEL: WidgetId = WidgetId::of("trust.cancel");
pub const TRUST_OK: WidgetId = WidgetId::of("trust.ok");
pub const TRUST_BODY: WidgetId = WidgetId::of("trust.body");

pub struct TrustPage {
    pub config: String,
    pub then: String,
    cancel: Button,
    ok: Button,
    scroll: ScrollState,
}

impl TrustPage {
    pub fn new(config: &str, then: &str) -> Self {
        Self {
            config: config.into(),
            then: then.into(),
            cancel: Button::secondary(TRUST_CANCEL, "Cancel"),
            ok: Button::primary(TRUST_OK, "Trust this file"),
            scroll: ScrollState::default(),
        }
    }

    fn lines(&self, w: &World) -> (Vec<Prop>, Vec<String>) {
        let short = w.location.short(&self.config);
        let mut facts = vec![Prop::new("File", short.clone())];
        let mut body = vec![];
        if let Some(cfg) = w.mise.configs.iter().find(|c| c.path == self.config) {
            facts.push(Prop::new(
                "Defines",
                format!(
                    "{} · {}",
                    plural(cfg.tasks.len(), "task", "tasks"),
                    plural(cfg.tools.len(), "tool", "tools")
                ),
            ));
            facts.push(
                Prop::new(
                    "Environment",
                    if cfg.env.is_empty() {
                        "no variables".into()
                    } else {
                        cfg.env.join(" · ")
                    },
                )
                .wrap(),
            );
            facts.push(Prop::new("Trust scope", "this exact file · not the folder, not descendants · renewed when the content changes"));
            facts.push(
                Prop::new(
                    "Provenance",
                    "committed in the project · last changed 3 d ago by the frontend team",
                )
                .tone(Tone::Muted),
            );
            facts.push(
                Prop::new(
                    "Then",
                    format!(
                        "re-resolve and run {}",
                        self.then.trim_start_matches("mise.task.")
                    ),
                )
                .tone(Tone::Muted),
            );
            for t in &cfg.tasks {
                body.push(format!("[tasks.{}]", t.name));
                body.push(format!("  run = \"{}\"", t.run));
                if !t.depends.is_empty() {
                    body.push(format!(
                        "  depends = [{}]",
                        t.depends
                            .iter()
                            .map(|d| format!("\"{d}\""))
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                body.push(format!("  description = \"{}\"", t.description));
            }
            if !cfg.tools.is_empty() {
                body.push("[tools]".into());
                for t in &cfg.tools {
                    body.push(format!("  {} = \"{}\"", t.name, t.requested));
                }
            }
            if !cfg.env.is_empty() {
                body.push("[env]".into());
                for e in &cfg.env {
                    body.push(format!("  {}", e.replacen('=', " = \"", 1) + "\""));
                }
            }
        } else if let Some(cfg) = w
            .custom_project
            .iter()
            .chain(w.custom_global.iter())
            .find(|c| c.path == self.config)
        {
            let dangerous = cfg
                .actions
                .iter()
                .filter(|a| a.danger == crate::domain::custom::Danger::Destructive)
                .count();
            facts.push(Prop::new(
                "Defines",
                format!(
                    "{} · {} destructive · {}",
                    plural(cfg.actions.len(), "custom action", "custom actions"),
                    dangerous,
                    plural(cfg.diagnostics.len(), "diagnostic", "diagnostics")
                ),
            ));
            facts.push(Prop::new("Digest", format!("sha256:{}", cfg.digest)).wrap());
            facts.push(Prop::new(
                "Trust scope",
                "the whole file by content digest · edited content is untrusted again · every action in it runs without a shell",
            ).wrap());
            facts.push(
                Prop::new(
                    "Provenance",
                    format!(
                        "{} · read from disk when the finder opened",
                        cfg.origin.label()
                    ),
                )
                .tone(Tone::Muted),
            );
            if let Some(a) = cfg.actions.iter().find(|a| a.id == self.then) {
                let cmd = crate::domain::exec::Command::from_vec(
                    a.argv.clone(),
                    &w.location.cwd,
                    &w.host.name,
                );
                for (k, v) in cmd.argv_facts() {
                    facts.push(Prop::new(&k, v).wrap());
                }
            }
            for d in &cfg.diagnostics {
                facts.push(Prop::new("Diagnostic", d.text()).tone(Tone::Warning).wrap());
            }
            facts.push(
                Prop::new(
                    "Then",
                    format!(
                        "run {} exactly as declared",
                        self.then.trim_start_matches("custom.")
                    ),
                )
                .tone(Tone::Muted),
            );
            body.extend(cfg.text.lines().map(str::to_owned));
        } else {
            facts.push(
                Prop::new(
                    "Defines",
                    "nothing readable · the file is missing or unreadable",
                )
                .tone(Tone::Error),
            );
            facts.push(
                Prop::new("Trust scope", "nothing can be trusted without content")
                    .tone(Tone::Muted),
            );
        }
        (facts, body)
    }
}

impl Screen for TrustPage {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        if cx.focus.is(TRUST_CANCEL) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                cx.go(Go::Pop);
                cx.status("Not trusted · the task did not run");
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cx.focus.is(TRUST_OK) {
            let (o, fired) = self.ok.on_key(key);
            if fired {
                cx.go(Go::Trusted {
                    config: self.config.clone(),
                    then: self.then.clone(),
                });
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if cx.focus.is(TRUST_BODY) => {
                self.scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') if cx.focus.is(TRUST_BODY) => {
                self.scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::Left | KeyCode::Char('h') if cx.focus.is(TRUST_OK) => {
                cx.focus.focus(TRUST_CANCEL);
                Outcome::Changed
            }
            KeyCode::Right | KeyCode::Char('l') if cx.focus.is(TRUST_CANCEL) => {
                cx.focus.focus(TRUST_OK);
                Outcome::Changed
            }
            KeyCode::Esc => {
                cx.go(Go::Pop);
                cx.status("Not trusted · the task did not run");
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, _pos: Position, _w: &mut World, cx: &mut Cx) -> Outcome {
        if id == TRUST_CANCEL {
            cx.go(Go::Pop);
            return Outcome::Changed;
        }
        if id == TRUST_OK {
            cx.go(Go::Trusted {
                config: self.config.clone(),
                then: self.then.clone(),
            });
            return Outcome::Changed;
        }
        if id == TRUST_BODY {
            cx.focus.focus(TRUST_BODY);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == TRUST_BODY {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn enter(&mut self, _w: &mut World, cx: &mut Cx) {
        cx.focus.focus(TRUST_CANCEL);
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let title = format!("Trust {}?", w.location.short(&self.config));
        buf.set_string(area.x + 1, area.y, &title, t.title());
        buf.set_string(area.x + 1, area.y + 1, "not trusted yet · review the exact definition, commands and environment before it runs", t.muted());
        let (facts, body) = self.lines(w);
        let facts_area = Rect::new(
            area.x + 1,
            area.y + 3,
            area.width.saturating_sub(2),
            facts.len() as u16,
        );
        let used = props::render(facts_area, buf, t, &facts, t.canvas);
        let buttons_y = area.bottom().saturating_sub(1);
        let body_y = facts_area.y + used + 1;
        let body_area = Rect::new(
            area.x,
            body_y,
            area.width,
            buttons_y.saturating_sub(body_y + 1),
        );
        let focused = ctx.interaction.focused(TRUST_BODY);
        let panel = Panel::framed(Some("Definition"))
            .focused(focused)
            .meta("exact content");
        let inner = panel.render(body_area, buf, t);
        self.scroll.set_content(body.len());
        self.scroll.set_viewport(inner.height as usize);
        ctx.control(TRUST_BODY, body_area, false);
        ctx.scrollable(TRUST_BODY, inner);
        for (k, i) in self.scroll.visible_range().enumerate() {
            let line = &body[i];
            let st = if line.starts_with('[') {
                t.primary()
            } else {
                t.secondary()
            };
            buf.set_string(
                inner.x,
                inner.y + k as u16,
                truncate(line, inner.width as usize),
                st,
            );
        }
        if self.scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(inner.right() - 1, inner.y, 1, inner.height),
                buf,
                ctx,
                TRUST_BODY,
                &self.scroll,
                focused,
            );
        }
        let widths = [self.cancel.width(), self.ok.width()];
        let rects = row_layout_right(
            Rect::new(area.x, buttons_y, area.width.saturating_sub(1), 1),
            &widths,
            2,
        );
        self.cancel.render(rects[0], buf, ctx, t.canvas);
        self.ok.render(rects[1], buf, ctx, t.canvas);
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        vec![
            hint("← →", "Choose"),
            hint("Enter", "Activate"),
            hint("Tab", "Next"),
            hint("Esc", "Cancel"),
        ]
    }

    fn crumb(&self, _w: &World) -> String {
        "Trust".into()
    }

    fn status(&self, _w: &World) -> StatusBits {
        StatusBits {
            center: Some(
                StatusItem::new("trust review · one exact file", Tone::Secondary).priority(7),
            ),
            right: vec![],
        }
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(TRUST_CANCEL)
    }
}

// ------------------------------------------------------------------ args

pub const ARGS_CANCEL: WidgetId = WidgetId::of("args.cancel");
pub const ARGS_RUN: WidgetId = WidgetId::of("args.run");
const ARG: WidgetId = WidgetId::of("args.field");

enum Field {
    Text(TextInput),
    Choice(Select),
}

pub struct ArgsPage {
    pub item: String,
    label: String,
    command: String,
    specs: Vec<ArgSpec>,
    fields: Vec<Field>,
    cancel: Button,
    run: Button,
}

impl ArgsPage {
    pub fn new(item: &str, w: &World) -> Self {
        let it: Option<Item> = w.items().into_iter().find(|i| i.id == item);
        let (label, command, specs) = match &it {
            Some(i) => (
                i.label.clone(),
                i.commands.first().cloned().unwrap_or_default(),
                i.args.clone(),
            ),
            None => (item.to_owned(), String::new(), vec![]),
        };
        let fields = specs
            .iter()
            .enumerate()
            .map(|(i, a)| {
                if a.options.is_empty() {
                    let mut f = TextInput::new(ARG.child(i), &a.name)
                        .value(&a.default)
                        .placeholder(&a.expected)
                        .required(a.required)
                        .help(&a.help);
                    if a.secret {
                        f = f.masked().reveal_tail(2);
                    }
                    Field::Text(f)
                } else {
                    let opts: Vec<&str> = a.options.iter().map(|s| s.as_str()).collect();
                    let sel = a.options.iter().position(|o| o == &a.default).unwrap_or(0);
                    Field::Choice(Select::new(ARG.child(i), &a.name, &opts, sel).help(&a.help))
                }
            })
            .collect();
        Self {
            item: item.into(),
            label,
            command,
            specs,
            fields,
            cancel: Button::subtle(ARGS_CANCEL, "Cancel"),
            run: Button::primary(ARGS_RUN, "Run"),
        }
    }

    fn values(&self) -> Vec<(String, String)> {
        self.specs
            .iter()
            .zip(&self.fields)
            .map(|(s, f)| {
                let v = match f {
                    Field::Text(t) => t.text().to_owned(),
                    Field::Choice(c) => c.value().to_owned(),
                };
                (s.name.clone(), v)
            })
            .collect()
    }

    fn preview(&self) -> String {
        let mut cmd = self.command.clone();
        for (k, v) in self.values() {
            if v.is_empty() {
                continue;
            }
            let shown = if self.specs.iter().any(|s| s.name == k && s.secret) {
                "••••".to_owned()
            } else {
                v
            };
            cmd = cmd.replace(&format!("<{k}>"), &shown);
            if !cmd.contains(&shown) {
                cmd.push_str(&format!(" --{k} {shown}"));
            }
        }
        cmd
    }

    fn submit(&mut self, cx: &mut Cx) -> Outcome {
        let mut ok = true;
        for (i, f) in self.fields.iter_mut().enumerate() {
            if let Field::Text(t) = f
                && !t.validate()
            {
                if ok {
                    cx.focus.focus(ARG.child(i));
                }
                ok = false;
            }
        }
        if !ok {
            cx.error("Fill the required fields first");
            return Outcome::Changed;
        }
        let args = self.values();
        cx.go(Go::Pop);
        cx.go(Go::Run {
            item: self.item.clone(),
            args,
        });
        Outcome::Changed
    }
}

impl Screen for ArgsPage {
    fn on_key(&mut self, key: &Key, _w: &mut World, cx: &mut Cx) -> Outcome {
        if key.ctrl() && key.code == KeyCode::Char('s') {
            return self.submit(cx);
        }
        for i in 0..self.fields.len() {
            let id = ARG.child(i);
            if !cx.focus.is(id) {
                continue;
            }
            match &mut self.fields[i] {
                Field::Text(t) => {
                    let (o, ev) = t.on_key(key);
                    match ev {
                        Some(InputEvent::CommittedTab { backward }) => {
                            if backward {
                                cx.focus.prev(cx.ring);
                            } else {
                                cx.focus.next(cx.ring);
                            }
                            return Outcome::Changed;
                        }
                        Some(InputEvent::Committed) => {
                            cx.focus.next(cx.ring);
                            return Outcome::Changed;
                        }
                        _ => {}
                    }
                    if o.consumed() {
                        return o;
                    }
                }
                Field::Choice(c) => {
                    let (o, _) = c.on_key(key);
                    if o.consumed() {
                        return o;
                    }
                }
            }
        }
        if cx.focus.is(ARGS_CANCEL) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                cx.go(Go::Pop);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cx.focus.is(ARGS_RUN) {
            let (o, fired) = self.run.on_key(key);
            if fired {
                return self.submit(cx);
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Esc => {
                cx.go(Go::Pop);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, _w: &mut World, cx: &mut Cx) -> Outcome {
        for i in 0..self.fields.len() {
            let fid = ARG.child(i);
            match &mut self.fields[i] {
                Field::Text(t) if fid == id => {
                    let was = cx.focus.is(fid);
                    cx.focus.focus(fid);
                    return t.on_click(pos, was);
                }
                Field::Choice(c) if c.owns(id) => {
                    cx.focus.focus(fid);
                    let (o, _) = c.on_click(id);
                    return o;
                }
                _ => {}
            }
        }
        if id == ARGS_CANCEL {
            cx.go(Go::Pop);
            return Outcome::Changed;
        }
        if id == ARGS_RUN {
            return self.submit(cx);
        }
        Outcome::Ignored
    }

    fn on_paste(&mut self, text: &str, _w: &mut World, cx: &mut Cx) -> Outcome {
        // the paste is typing for the focused field: it starts editing the
        // way a key would; a button or choice takes no paste
        for (i, f) in self.fields.iter_mut().enumerate() {
            if let Field::Text(t) = f
                && cx.focus.is(ARG.child(i))
            {
                return t.on_paste(text);
            }
        }
        Outcome::Ignored
    }

    fn enter(&mut self, _w: &mut World, cx: &mut Cx) {
        cx.focus.focus(ARG.child(0));
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, _w: &World) {
        let t = ctx.theme;
        let title = format!("{} · arguments", self.label);
        buf.set_string(area.x + 1, area.y, &title, t.title());
        buf.set_string(
            area.x + 1,
            area.y + 1,
            "field name, expected value, default and validation · secrets stay masked",
            t.muted(),
        );
        let col_w = area.width.saturating_sub(2).min(56);
        let mut y = area.y + 3;
        for f in &mut self.fields {
            let r = Rect::new(area.x, y, col_w, 3);
            match f {
                Field::Text(t2) => t2.render(r, buf, ctx, t.canvas),
                Field::Choice(c) => c.render(r, buf, ctx, t.canvas),
            }
            y += 4;
        }
        // command preview card, then the buttons one blank row under it:
        // anchors stay with the content, not the page edge
        let card_y = y;
        let buttons_y = (card_y + 4).min(area.bottom().saturating_sub(1));
        if card_y + 3 < buttons_y {
            let card = Rect::new(area.x, card_y, area.width.saturating_sub(1), 3);
            let panel = Panel::card(Some("Command"));
            let inner = panel.render(card, buf, t);
            buf.set_string(
                inner.x,
                inner.y,
                truncate(&self.preview(), inner.width as usize),
                t.secondary().bg(panel.bg(t)),
            );
        }
        let widths = [self.cancel.width(), self.run.width()];
        let rects = row_layout_right(
            Rect::new(area.x, buttons_y, area.width.saturating_sub(1), 1),
            &widths,
            2,
        );
        self.cancel.render(rects[0], buf, ctx, t.canvas);
        self.run.render(rects[1], buf, ctx, t.canvas);
        let _ = heading;
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if self.is_editing() {
            return vec![
                hint("Enter", "Commit"),
                hint("Tab", "Next field"),
                hint("Esc", "Revert"),
            ];
        }
        vec![
            hint("Enter", "Edit"),
            hint("Tab", "Next"),
            hint("Ctrl+S", "Run"),
            hint("Esc", "Cancel"),
        ]
    }

    fn crumb(&self, _w: &World) -> String {
        "Arguments".into()
    }

    fn is_editing(&self) -> bool {
        self.fields
            .iter()
            .any(|f| matches!(f, Field::Text(t) if t.editing))
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(ARG.child(0))
    }
}
