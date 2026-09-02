//! Launch cockpit: the 11-stage rail advancing on deterministic ticks,
//! identity, credentials origin, activity, atmosphere, and the build log,
//! failure, container-info and quit overlays.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::{Theme, Tone};
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{truncate, width};
use junie_tui::widgets::button::Button;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::progress::spinner_frame;
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::Segment;
use junie_tui::widgets::steps::{Step, StepRail, StepState};
use junie_tui::widgets::viewport::{Span, TextViewport};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::modals::{InfoDialog, InfoResult, modal_frame};
use super::{Cx, Go, Modal, ModalResult, ModalTag, Screen};
use crate::domain::account::AccountId;
use crate::domain::agent::Agent;
use crate::domain::fixtures::PrecedenceLevel;
use crate::domain::instance::{Instance, InstanceStatus};
use crate::domain::workspace::WorkspaceId;
use crate::scenario::Motion;
use crate::sim::launch::{LaunchEvent, LaunchPlan, LaunchRun, Stage};
use crate::sim::world::World;

pub const RAIL: WidgetId = WidgetId::of("cockpit.rail");
pub const LOG: WidgetId = WidgetId::of("cockpit.log");
pub const CHIP_CONTAINER: WidgetId = WidgetId::of("cockpit.chip.container");
pub const CHIP_ACTIVITY: WidgetId = WidgetId::of("cockpit.chip.activity");
pub const CHIP_RUN: WidgetId = WidgetId::of("cockpit.chip.run");

pub struct CockpitScreen {
    pub run: LaunchRun,
    pub workspace: Option<WorkspaceId>,
    pub role: String,
    pub agent: Agent,
    pub account: Option<AccountId>,
    pub rail: StepRail,
    pub activity: String,
    pub log: TextViewport,
    pub log_open: bool,
    pub container: Option<String>,
    pub credentials: Option<(String, String, Tone)>,
    pub tick: u64,
    pub motion: Motion,
    pub frozen_at: Option<u64>,
    pub instance_id: String,
    pub handoff_requested: bool,
    pub failure_shown: bool,
    log_area: Rect,
    pub debug: bool,
    pub credential_hold: bool,
    pub target_label: String,
}

impl CockpitScreen {
    pub fn new(
        w: &World,
        workspace: Option<WorkspaceId>,
        role: String,
        agent: Agent,
        account: Option<AccountId>,
        plan: LaunchPlan,
        motion: Motion,
    ) -> Self {
        let wsname = workspace
            .and_then(|id| w.workspace(id))
            .map(|x| x.name.clone())
            .unwrap_or_else(|| w.cwd.rsplit('/').next().unwrap_or("directory").to_owned());
        let instance_id = w.new_instance_id();
        let suffix = instance_id.trim_start_matches("jk-").to_owned();
        let container = format!("jackin-{wsname}-{suffix}");
        let run_id = format!("run-{}-{suffix}", crate::clock::Clock::stamp(w.now_secs()).replace([' ', ':'], "-").replace('-', "")[..12].to_owned());
        let run = LaunchRun::new(plan, agent, &container, &run_id);
        let steps = Stage::ALL.iter().map(|s| Step::new(s.label())).collect();
        let mut log = TextViewport::new(LOG).wrap(true).max_lines(5_000);
        log.follow = true;
        let target_label = match workspace.and_then(|id| w.workspace(id)) {
            Some(x) => format!("into workspace {}", x.name),
            None => format!("in directory {}", w.tilde(&w.cwd)),
        };
        Self {
            run,
            workspace,
            role,
            agent,
            account,
            rail: StepRail::new(RAIL, steps).selectable(false),
            activity: "Preparing launch…".into(),
            log,
            log_open: false,
            container: None,
            credentials: None,
            tick: 0,
            motion,
            frozen_at: None,
            instance_id,
            handoff_requested: false,
            failure_shown: false,
            log_area: Rect::ZERO,
            debug: false,
            credential_hold: false,
            target_label,
        }
    }

    /// Fast-forward to a fixture tick (paused captures).
    pub fn seek(&mut self, ticks: u64, w: &mut World, cx: &mut Cx) {
        for _ in 0..ticks {
            self.step(w, cx);
        }
    }

    fn credential_origin(&self, w: &World) -> (String, String, Tone) {
        match self.account.as_ref().and_then(|id| w.accounts.get(id)) {
            Some(a) => {
                let why = w
                    .account_for(a.provider, self.workspace.and_then(|id| w.workspace(id)), Some(&self.role), None)
                    .level;
                let origin = format!("{} · {} · {}", a.title(), a.source.origin_label(), a.source.safe_detail());
                let (val, tone) = match a.validation.level() {
                    Some(l) => (format!("{} · {}", l.label(), why.label()), Tone::Secondary),
                    None => (format!("{} · {}", a.validation.label(), why.label()), Tone::Warning),
                };
                (origin, val, tone)
            }
            None => (
                format!("host profile · {} sync", self.agent.label()),
                format!("{} · no registered account", PrecedenceLevel::None.label()),
                Tone::Warning,
            ),
        }
    }

    fn step(&mut self, w: &mut World, cx: &mut Cx) -> bool {
        if self.run.is_terminal() || self.handoff_requested {
            return false;
        }
        self.tick += 1;
        let events = self.run.advance();
        let mut changed = !events.is_empty();
        for ev in events {
            match ev {
                LaunchEvent::StageChanged(stage, state) => {
                    self.rail.set_state(stage.index(), state);
                    if state == StepState::Done {
                        self.rail.set_meta(stage.index(), Some(format!("{:.1} s", self.run.durations[stage.index()] as f64 * 0.033 * 3.0)));
                    }
                    if state == StepState::Running && stage == Stage::Credentials {
                        let (o, v, tone) = self.credential_origin(w);
                        self.credentials = Some((o, v, tone));
                    }
                }
                LaunchEvent::Activity(a) => self.activity = a,
                LaunchEvent::BuildLine(l) => self.log.push(ansi_line(&l)),
                LaunchEvent::ContainerReady(c) => self.container = Some(c),
                LaunchEvent::CredentialsResolved { .. } => {
                    if let Some((_, v, tone)) = self.credentials.as_mut() {
                        *v = format!("resolved in memory · discarded after injection · {v}");
                        *tone = Tone::Secondary;
                    }
                }
                LaunchEvent::CredentialError { message } => {
                    self.credential_hold = true;
                    if let Some((_, v, tone)) = self.credentials.as_mut() {
                        *v = message.clone();
                        *tone = Tone::Error;
                    }
                    let mut d = Dialog::destructive(
                        WidgetId::of("cockpit.cred"),
                        "Credential source unavailable",
                        &format!("{message}. The launch is paused at the Credentials stage; nothing was injected."),
                        "Cancel launch",
                    );
                    d.actions = vec![
                        Button::secondary(WidgetId::of("cockpit.cred").sub("retry"), "Retry"),
                        Button::secondary(WidgetId::of("cockpit.cred").sub("plain"), "Enter plain text instead"),
                        Button::danger(WidgetId::of("cockpit.cred").sub("cancel"), "Cancel launch"),
                    ];
                    d.cancel_index = Some(2);
                    d.initial_focus = d.actions[0].id;
                    d.width = 70;
                    cx.open(Modal::Dialog(d), ModalTag::new("cred"));
                }
                LaunchEvent::Failed(f) => {
                    self.frozen_at = Some(self.tick);
                    self.activity = format!("{} failed", f.stage.label());
                    self.log_open = false;
                    let record = self.instance_record(w, InstanceStatus::FailedSetup);
                    w.instances.push(record);
                    w.sync_arbiter();
                    self.open_failure(w, cx);
                }
                LaunchEvent::Ready => {
                    self.activity = "Hardline open · handing off to the Capsule".into();
                    // the instance becomes durable now
                    let record = self.instance_record(w, InstanceStatus::Running);
                    let id = record.id.clone();
                    w.instances.push(record);
                    let now = w.now_ms();
                    let wsname = self.target_label.rsplit(' ').next().unwrap_or("workspace").trim_start_matches("~/").to_owned();
                    let wsname = self
                        .workspace
                        .and_then(|i| w.workspace(i))
                        .map(|x| x.name.clone())
                        .unwrap_or(wsname);
                    let mut d = crate::sim::pty::Daemon::new(&id, &wsname, now);
                    d.new_tab(Some(self.agent), self.account.clone(), now, false);
                    w.daemons.insert(id.clone(), d);
                    crate::domain::fixtures::refresh_snapshots(w);
                    w.sync_arbiter();
                    self.handoff_requested = true;
                    cx.go(Go::Attach {
                        instance: id,
                        pane: None,
                    });
                }
            }
            changed = true;
        }
        changed
    }

    fn instance_record(&self, w: &World, status: InstanceStatus) -> Instance {
        let now = w.now_secs();
        let wsname = self
            .workspace
            .and_then(|i| w.workspace(i))
            .map(|x| x.name.clone())
            .unwrap_or_else(|| w.cwd.rsplit('/').next().unwrap_or("directory").to_owned());
        let mut i = Instance {
            id: self.instance_id.clone(),
            container: format!("jackin-{wsname}"),
            workspace: self.workspace,
            workdir: format!("/workspace/{wsname}"),
            role: self.role.clone(),
            agent: self.agent,
            status,
            created_secs: now,
            last_seen_secs: now,
            run_id: self.run.run_id.clone(),
            sessions: Ok(vec![]),
            daemon: crate::domain::instance::DaemonSnapshot::Unavailable,
            branch: None,
            pr: None,
            default_branch: "main".into(),
            uncommitted: 0,
            unpushed: 0,
        };
        if let Some(r) = w.github.iter().find(|r| r.full_name.ends_with(&wsname)) {
            i.branch = r.branches.get(1).cloned();
        }
        i
    }

    fn open_failure(&mut self, w: &World, cx: &mut Cx) {
        let Some(f) = self.run.failure.clone() else { return };
        self.failure_shown = true;
        let props = vec![
            Prop::new("Stage", f.stage.label()).tone(Tone::Error),
            Prop::new("Run id", self.run.run_id.clone()).copyable(),
            Prop::new("Next step", f.next_step.clone()).wrap(),
            Prop::new("Container", self.container.clone().unwrap_or("not created".into())),
        ];
        let title = match f.stage {
            Stage::DerivedImage => "Docker build failed".to_owned(),
            Stage::Credentials => "Credential check failed".to_owned(),
            _ => "Launch failed".to_owned(),
        };
        let d = InfoDialog::new(WidgetId::of("cockpit.failure"), &title, props)
            .error()
            .intro(vec![(f.summary.clone(), Tone::Normal), (format!("Loading {} {}", self.role, self.target_label), Tone::Muted)])
            .detail(f.detail.clone())
            .meta("run id is the only copyable value")
            .width(70);
        let _ = w;
        cx.open(Modal::Info(d), ModalTag::new("failure"));
    }

    fn open_info(&mut self, w: &World, cx: &mut Cx) {
        let mut props = vec![
            Prop::new("Target", format!("{} {}", self.role, self.target_label)),
            Prop::new("Role", self.role.clone()),
            Prop::new("Agent", format!("{} · account {}", self.agent.label(), self.account.as_ref().and_then(|id| w.accounts.get(id)).map(|a| a.title()).unwrap_or("host profile".into()))),
            Prop::new("Run id", self.run.run_id.clone()).copyable(),
            Prop::new("jackin", "0.6.4 · preview"),
        ];
        if let Some(c) = &self.container {
            props.insert(0, Prop::new("Container", c.clone()).copyable());
        }
        if self.debug {
            props.push(Prop::new("Telemetry", format!("run {} -> otlp://collector.internal:4317", self.run.run_id)));
        }
        let d = InfoDialog::new(WidgetId::of("cockpit.info"), "Debug info", props).meta("read-only");
        cx.open(Modal::Info(d), ModalTag::new("info"));
    }

    fn open_quit(&mut self, cx: &mut Cx) {
        let d = Dialog::destructive(
            WidgetId::of("cockpit.quit"),
            "Exit jackin❯?",
            "Exiting force-stops the launch immediately. Partially prepared resources are cleaned up on the next launch.",
            "Exit",
        );
        cx.open(Modal::Dialog(d), ModalTag::new("quit"));
    }

    fn cancel(&mut self, w: &mut World, cx: &mut Cx) {
        if self.run.is_terminal() {
            return;
        }
        self.run.cancel();
        self.frozen_at = Some(self.tick);
        self.activity = "Launch cancelled".into();
        let record = self.instance_record(w, InstanceStatus::FailedSetup);
        w.instances.push(record);
        w.sync_arbiter();
        cx.status("Launch cancelled · nothing was attached");
        cx.go(Go::LaunchFailedAck {
            instance: Some(self.instance_id.clone()),
        });
    }

    fn draw_log(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let active = !self.run.is_terminal() && self.rail.frontier().is_some_and(|i| i <= Stage::DerivedImage.index());
        let title = if active { "Docker build · building…" } else { "Docker build" };
        let n = self.log.len();
        let meta = if self.log.follow { format!("{n} lines · following") } else { format!("{n} lines") };
        let w = screen.width.saturating_sub(6).min(110);
        let h = screen.height.saturating_sub(4);
        let (_, inner) = modal_frame(screen, buf, ctx, w, h, title, Some(&meta), true);
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let body = Rect::new(inner.x, inner.y, inner.width, inner.height.saturating_sub(2));
        self.log_area = body;
        if self.log.is_empty() {
            buf.set_string(body.x, body.y, "(waiting for docker build output…)", t.muted().bg(bg));
        } else {
            self.log.render(body, buf, ctx, bg);
        }
        let hint = "↑↓ Scroll · PgUp PgDn Page · End Follow · Esc Close";
        buf.set_string(inner.x, inner.bottom() - 1, truncate(hint, inner.width as usize), t.faint().bg(bg));
        ctx.hits.register(LOG, body);
        ctx.hits.register(junie_tui::widgets::scrollbar::id_for(LOG), Rect::new(body.right() - 1, body.y, 1, body.height));
    }
}

/// Fixture ANSI-like markup → styled spans: `#N` step prefixes muted,
/// `DONE`/`CACHED` secondary, `warning` amber, `error` red.
pub fn ansi_line(l: &str) -> Vec<Span> {
    let mut spans = vec![];
    if let Some(rest) = l.strip_prefix('#') {
        let end = rest.find(' ').unwrap_or(rest.len());
        spans.push(Span::muted(format!("#{}", &rest[..end])));
        let body = &rest[end..];
        let tone = if body.contains("DONE") || body.contains("CACHED") {
            Tone::Secondary
        } else if body.contains("warning") {
            Tone::Warning
        } else if body.contains("error") || body.contains("Error") {
            Tone::Error
        } else {
            Tone::Normal
        };
        spans.push(Span::new(body.to_owned(), tone));
    } else {
        spans.push(Span::plain(l.to_owned()));
    }
    spans
}

fn wrapped_arrow(buf: &mut Buffer, area: Rect, t: &Theme) {
    // continuation glyph for wrapped lines is the viewer's own wrap; mark the
    // left edge of the log body once
    let _ = (buf, area, t);
}

impl Screen for CockpitScreen {
    fn enter(&mut self, _w: &mut World, cx: &mut Cx) {
        cx.focus.focus(RAIL);
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(RAIL)
    }

    fn animating(&self, _w: &World) -> bool {
        !self.run.is_terminal() || self.frozen_at.is_none()
    }

    fn on_tick(&mut self, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.motion == Motion::Paused {
            return Outcome::Ignored;
        }
        if self.motion == Motion::Reduced {
            // reduced motion: the pipeline still advances, atmosphere is static
            let mut changed = false;
            for _ in 0..3 {
                changed |= self.step(w, cx);
            }
            return if changed { Outcome::Changed } else { Outcome::Consumed };
        }
        let changed = self.step(w, cx);
        if changed || self.frozen_at.is_none() {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if key.ctrl_char('c') {
            // hard abort: immediate, no cleanup wait
            cx.status("Aborted · terminal restored, cleanup deferred");
            self.run.cancel();
            let record = self.instance_record(w, InstanceStatus::FailedSetup);
            w.instances.push(record);
            w.sync_arbiter();
            cx.go(Go::LaunchFailedAck {
                instance: Some(self.instance_id.clone()),
            });
            return Outcome::Changed;
        }
        if key.ctrl_char('q') {
            self.open_quit(cx);
            return Outcome::Changed;
        }
        if self.log_open {
            match key.code {
                KeyCode::Esc | KeyCode::Char('b') | KeyCode::Char('q') => {
                    self.log_open = false;
                    cx.focus.focus(RAIL);
                    return Outcome::Changed;
                }
                _ => {
                    let (o, _) = self.log.on_key(key);
                    return o.or(Outcome::Consumed);
                }
            }
        }
        match key.code {
            KeyCode::Char('b') | KeyCode::Enter if key.plain() => {
                if self.log.is_empty() && self.rail.frontier().is_some_and(|i| i < Stage::DerivedImage.index()) {
                    cx.status("No build output yet · the derived image stage has not started");
                    return Outcome::Changed;
                }
                self.log_open = true;
                self.log.set_follow(true);
                cx.focus.focus(LOG);
                Outcome::Changed
            }
            KeyCode::Char('i') if key.plain() => {
                self.open_info(w, cx);
                Outcome::Changed
            }
            KeyCode::Char('c') if key.plain() => {
                if self.run.is_terminal() {
                    return Outcome::Consumed;
                }
                let d = Dialog::destructive(
                    WidgetId::of("cockpit.cancel"),
                    "Cancel the launch?",
                    "The pipeline stops at its current stage and the partially prepared instance is marked failed setup. Nothing is attached.",
                    "Cancel launch",
                );
                cx.open(Modal::Dialog(d), ModalTag::new("cancel"));
                Outcome::Changed
            }
            KeyCode::Char('d') if key.plain() => {
                self.debug = !self.debug;
                Outcome::Changed
            }
            KeyCode::Esc if self.run.failure.is_some() && !self.failure_shown => {
                self.open_failure(w, cx);
                Outcome::Changed
            }
            KeyCode::Esc | KeyCode::Enter if self.run.blocked_at.is_some() => {
                cx.status("Blocked stage is a modeled state · cancel with c or quit with Ctrl+Q");
                Outcome::Changed
            }
            _ => Outcome::Consumed,
        }
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if id == CHIP_ACTIVITY && !self.log.is_empty() {
            self.log_open = true;
            cx.focus.focus(LOG);
            return Outcome::Changed;
        }
        if id == CHIP_CONTAINER || id == CHIP_RUN {
            self.open_info(w, cx);
            return Outcome::Changed;
        }
        if id == LOG {
            return self.log.on_click(pos);
        }
        if id == junie_tui::widgets::scrollbar::id_for(LOG) {
            return self.log.on_scrollbar(pos);
        }
        Outcome::Consumed
    }

    fn on_drag(&mut self, pressed: WidgetId, pos: Position, _w: &mut World) -> Outcome {
        if pressed == junie_tui::widgets::scrollbar::id_for(LOG) {
            return self.log.on_scrollbar(pos);
        }
        if pressed == LOG {
            return self.log.on_drag(pos);
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if self.log_open && (id == LOG || id == junie_tui::widgets::scrollbar::id_for(LOG) || id == WidgetId::of("modal.surface")) {
            return self.log.on_wheel(delta);
        }
        Outcome::Ignored
    }

    fn on_modal(&mut self, tag: &ModalTag, result: ModalResult, w: &mut World, cx: &mut Cx) -> Outcome {
        match (tag.kind, result) {
            ("failure", ModalResult::Info(InfoResult::Copy(v))) => {
                cx.copy(v);
                cx.status("Run id copied");
                // keep the failure acknowledged only on explicit close
                self.open_failure(w, cx);
                Outcome::Changed
            }
            ("failure", _) => {
                cx.go(Go::LaunchFailedAck {
                    instance: Some(self.instance_id.clone()),
                });
                Outcome::Changed
            }
            ("info", ModalResult::Info(InfoResult::Copy(v))) => {
                cx.copy(v);
                Outcome::Changed
            }
            ("quit", ModalResult::Dialog { action: Some(1), .. }) => {
                self.run.cancel();
                let record = self.instance_record(w, InstanceStatus::FailedSetup);
                w.instances.push(record);
                w.sync_arbiter();
                cx.go(Go::Quit);
                Outcome::Changed
            }
            ("cancel", ModalResult::Dialog { action: Some(1), .. }) => {
                self.cancel(w, cx);
                Outcome::Changed
            }
            ("cred", ModalResult::Dialog { action, .. }) => match action {
                Some(0) => {
                    // retry: the fixture 1Password session unlocks
                    w.op.session = crate::sim::onepassword::OpSession::SignedIn;
                    self.credential_hold = false;
                    self.run.retry_credentials();
                    if let Some((_, v, tone)) = self.credentials.as_mut() {
                        *v = "retrying after unlock…".into();
                        *tone = Tone::Secondary;
                    }
                    cx.status("1Password unlocked · resolving the reference again");
                    Outcome::Changed
                }
                Some(1) => {
                    self.credential_hold = false;
                    self.account = None;
                    self.run.retry_credentials();
                    if let Some((o, v, tone)) = self.credentials.as_mut() {
                        *o = format!("plain text · masked API key · {}", crate::domain::account::masked("Q1zx"));
                        *v = "typed once for this launch · never stored".into();
                        *tone = Tone::Secondary;
                    }
                    cx.status("Using a plain-text key for this launch only");
                    Outcome::Changed
                }
                _ => {
                    self.cancel(w, cx);
                    Outcome::Changed
                }
            },
            _ => Outcome::Changed,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        fill(buf, area, t.base());
        // layout: identity block (4 rows), rail, activity, chips
        let rail_h = (Stage::ALL.len() as u16).min(area.height.saturating_sub(9));
        let rail_w = 44u16.min(area.width.saturating_sub(4));
        let rail_x = area.x + (area.width.saturating_sub(rail_w)) / 2;
        let ident_y = area.y + 1;
        let rail_y = ident_y + 5;
        let rail = Rect::new(rail_x, rail_y, rail_w, rail_h);
        // atmosphere behind everything, excluding the rail and identity block
        let exclude = [
            Rect::new(rail_x.saturating_sub(2), ident_y, rail_w + 4, rail_h + 5 + 1),
            Rect::new(area.x, area.bottom().saturating_sub(3), area.width, 3),
        ];
        let running = !self.run.is_terminal() && self.frozen_at.is_none();
        let t_local = match self.motion {
            Motion::Reduced => 60,
            _ => self.frozen_at.unwrap_or(self.tick),
        };
        crate::rain::paint_atmosphere(buf, area, &exclude, t_local, running, self.frozen_at.is_some(), t);
        // identity
        let ws_name = self.target_label.clone();
        let head = if self.rail.frontier().is_none() && self.tick == 0 {
            "Preparing launch…".to_owned()
        } else {
            format!("Loading {} {ws_name}", self.role)
        };
        let center = |buf: &mut Buffer, y: u16, text: &str, style: Style| {
            let n = width(text) as u16;
            let x = area.x + area.width.saturating_sub(n) / 2;
            buf.set_string(x, y, truncate(text, area.width as usize), style);
        };
        center(buf, ident_y, " jackin❯ ", Style::new().fg(t.text_on_accent).bg(t.accent).add_modifier(Modifier::BOLD));
        center(buf, ident_y + 1, &head, t.title());
        let acc = self
            .account
            .as_ref()
            .and_then(|id| w.accounts.get(id))
            .map(|a| a.title())
            .unwrap_or("host profile".into());
        let why = self
            .account
            .as_ref()
            .and_then(|id| w.accounts.get(id))
            .map(|a| w.account_for(a.provider, self.workspace.and_then(|i| w.workspace(i)), Some(&self.role), None).level.label())
            .unwrap_or("no registered account");
        center(
            buf,
            ident_y + 2,
            &format!("{} · {} · account {acc} ({why})", self.agent.label(), self.agent.provider().label()),
            t.secondary(),
        );
        let (done, skipped) = self.run.counts();
        let frontier = self
            .rail
            .frontier()
            .map(|i| format!("stage {} of {} · {}", i + 1, Stage::ALL.len(), Stage::ALL[i].label()))
            .unwrap_or_else(|| if self.run.done { "all stages complete".into() } else { "stopped".into() });
        center(
            buf,
            ident_y + 3,
            &format!("{frontier} · {done} done · {skipped} skipped"),
            t.muted(),
        );
        // rail on a canvas strip so rain never shows through
        fill(buf, Rect::new(rail.x - 1, rail.y, rail.width + 2, rail.height), t.base());
        self.rail.render(rail, buf, ctx, t.canvas);
        // credentials line under the rail when relevant
        let mut y = rail.bottom() + 1;
        if let Some((origin, val, tone)) = &self.credentials
            && y + 1 < area.bottom().saturating_sub(3)
        {
            let line = format!("credentials  {origin}");
            let x = rail.x.saturating_sub(4);
            let wdt = area.right().saturating_sub(x + 2) as usize;
            fill(buf, Rect::new(area.x, y, area.width, 2), t.base());
            buf.set_string(x, y, truncate(&line, wdt), t.muted());
            buf.set_string(x + 13, y + 1, truncate(val, wdt.saturating_sub(13)), Style::new().fg(t.tone(*tone)));
            y += 3;
        }
        let _ = y;
        // bottom chrome: activity chip (clickable → build log), container, run id
        let ay = area.bottom().saturating_sub(2);
        fill(buf, Rect::new(area.x, ay - 1, area.width, 3), t.base());
        let spinner = if running { format!("{} ", spinner_frame(self.tick)) } else if self.run.failure.is_some() { "! ".into() } else { "".into() };
        let activity = format!("{spinner}{}", self.activity);
        let act_style = if self.run.failure.is_some() { t.error_fg() } else { t.secondary() };
        let ax = area.x + 1;
        buf.set_string(ax, ay, truncate(&activity, area.width.saturating_sub(2) as usize), act_style);
        if !self.log.is_empty() {
            let lines = format!("{} lines · b build log", self.log.len());
            buf.set_string(ax, ay + 1, truncate(&lines, area.width.saturating_sub(2) as usize), t.faint());
            ctx.clickable(CHIP_ACTIVITY, Rect::new(ax, ay, width(&activity) as u16, 2));
        }
        let mut rx = area.right().saturating_sub(1);
        if self.debug {
            let chip = format!(" {} ", self.run.run_id);
            let cw = width(&chip) as u16;
            rx = rx.saturating_sub(cw);
            buf.set_string(rx, ay, &chip, Style::new().fg(t.warning));
            ctx.clickable(CHIP_RUN, Rect::new(rx, ay, cw, 1));
            rx = rx.saturating_sub(2);
        }
        if let Some(c) = &self.container {
            let chip = format!(" {c} ");
            let cw = width(&chip) as u16;
            if rx > cw + width(&activity) as u16 + 4 {
                rx = rx.saturating_sub(cw);
                let hovered = ctx.interaction.hovered(CHIP_CONTAINER);
                buf.set_string(rx, ay, &chip, if hovered { t.primary().bg(t.surface_elevated) } else { t.secondary() });
                ctx.clickable(CHIP_CONTAINER, Rect::new(rx, ay, cw, 1));
            }
        }
        wrapped_arrow(buf, area, t);
        ctx.control(RAIL, Rect::new(rail.x, rail.y, 1, 1), false);
        if self.log_open {
            self.draw_log(area, buf, ctx);
        }
    }

    fn hints(&self, _focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if self.log_open {
            return vec![hint("↑↓", "Scroll"), hint("End", "Follow"), hint("Esc", "Close log")];
        }
        let mut v = vec![];
        if !self.log.is_empty() {
            v.push(hint("b", "Build log"));
        }
        v.push(hint("i", "Container info"));
        if !self.run.is_terminal() {
            v.push(hint("c", "Cancel"));
        }
        v.push(hint("Ctrl+Q", "Quit"));
        v.push(hint("Ctrl+C", "Abort"));
        v
    }

    fn crumb(&self, _w: &World) -> String {
        format!("Launch › {} › {}", self.target_label.rsplit(' ').next().unwrap_or(""), self.role)
    }

    fn strip_right(&self, _w: &World) -> Vec<Segment> {
        let (done, _) = self.run.counts();
        vec![Segment::new(format!("{done}/{} stages", Stage::ALL.len()), Tone::Muted).priority(5)]
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        cx.status("Esc does nothing here · c cancels, Ctrl+Q quits");
        Outcome::Changed
    }
}
