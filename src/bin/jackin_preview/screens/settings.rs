//! Global Settings: General · Mounts · Environments · Auth · Trust with the
//! Editor's stage grammar, a `global` scope column on every scoped row and
//! the same preview → async save flow.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width};
use junie_tui::widgets::button::{Button, row_layout_right};
use junie_tui::widgets::choice::Checkbox;
use junie_tui::widgets::dialog::Dialog;
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::progress::spinner_frame;
use junie_tui::widgets::props::Prop;
use junie_tui::widgets::segments::Segment;
use junie_tui::widgets::tabs::{TabEvent, TabItem, Tabs};
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::Modifier;

use super::config::{ConfigTabs, Doc, Scope, Tab as CfgTab};
use super::modals::InfoDialog;
use super::{Cx, Go, Modal, ModalResult, ModalTag, Screen, plural};
use crate::domain::workspace::RoleName;
use crate::sim::world::{GlobalConfig, Msg, World};

pub const TABS: WidgetId = WidgetId::of("settings.tabs");
pub const COAUTHOR: WidgetId = WidgetId::of("settings.coauthor");
pub const DCO: WidgetId = WidgetId::of("settings.dco");
pub const TRUST: WidgetId = WidgetId::of("settings.trust");
pub const CANCEL: WidgetId = WidgetId::of("settings.cancel");
pub const SAVE: WidgetId = WidgetId::of("settings.save");
const BASE: WidgetId = WidgetId::of("settings.cfg");

const TAB_NAMES: [&str; 5] = ["General", "Mounts", "Environments", "Auth", "Trust"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StTab {
    General,
    Mounts,
    Environments,
    Auth,
    Trust,
}

impl StTab {
    const ALL: [StTab; 5] = [
        StTab::General,
        StTab::Mounts,
        StTab::Environments,
        StTab::Auth,
        StTab::Trust,
    ];
    fn cfg(self) -> Option<CfgTab> {
        match self {
            StTab::Mounts => Some(CfgTab::Mounts),
            StTab::Environments => Some(CfgTab::Environments),
            StTab::Auth => Some(CfgTab::Auth),
            _ => None,
        }
    }
}

pub struct SettingsScreen {
    pub original: GlobalConfig,
    pub pending: GlobalConfig,
    pub cfg: ConfigTabs,
    tabs: Tabs,
    tab: StTab,
    coauthor: Checkbox,
    dco: Checkbox,
    trust_cursor: usize,
    cancel: Button,
    save: Button,
    saving: bool,
    row_status: Option<String>,
}

impl SettingsScreen {
    pub fn new(w: &World) -> Self {
        let original = w.global.clone();
        let pending = original.clone();
        let registry: Vec<RoleName> = w.roles.iter().map(|r| r.name.clone()).collect();
        let cfg = ConfigTabs::new(
            Scope::Global,
            Doc::from_global(&original),
            Doc::from_global(&pending),
            BASE,
            vec![],
            registry,
        );
        Self {
            coauthor: Checkbox::new(
                COAUTHOR,
                "Add Co-authored-by trailer",
                pending.coauthor_trailer,
            ),
            dco: Checkbox::new(DCO, "Sign off commits (DCO)", pending.dco_signoff),
            original,
            pending,
            cfg,
            tabs: Tabs::new(TABS, &TAB_NAMES),
            tab: StTab::General,
            trust_cursor: 0,
            cancel: Button::subtle(CANCEL, "Cancel"),
            save: Button::primary(SAVE, "Save…"),
            saving: false,
            row_status: None,
        }
    }

    fn sync_pending(&mut self) {
        self.pending.coauthor_trailer = self.coauthor.checked;
        self.pending.dco_signoff = self.dco.checked;
        self.cfg.pending.apply_to_global(&mut self.pending);
    }

    pub fn change_count(&self) -> usize {
        self.pending.change_count(&self.original)
    }

    fn general_dirty(&self) -> bool {
        self.pending.coauthor_trailer != self.original.coauthor_trailer
            || self.pending.dco_signoff != self.original.dco_signoff
    }

    fn trust_dirty(&self) -> bool {
        self.pending.trust != self.original.trust
    }

    fn set_tab(&mut self, i: usize, cx: &mut Cx) {
        self.tab = StTab::ALL[i.min(4)];
        self.tabs.set_active(i);
        cx.focus.focus(TABS);
    }

    fn body_focus(&self) -> WidgetId {
        match self.tab {
            StTab::General => COAUTHOR,
            StTab::Trust => TRUST,
            t => self.cfg.list_id(t.cfg().unwrap()),
        }
    }

    fn open_preview(&mut self, w: &World, cx: &mut Cx) {
        self.sync_pending();
        let n = self.change_count();
        if n == 0 {
            cx.status("Nothing to save");
            return;
        }
        if self.cfg.tab_error(CfgTab::Environments) {
            self.set_tab(2, cx);
            cx.error("An environment key is invalid");
            return;
        }
        let blockers = self.cfg.blockers();
        let sensitive: Vec<String> = self
            .cfg
            .pending
            .mounts
            .iter()
            .filter(|m| is_sensitive(m.source_label()))
            .map(|m| m.source_label().to_owned())
            .collect();
        let mut facts = vec![
            Prop::new(
                "Scope",
                format!(
                    "global config · {}",
                    w.tilde(&format!("{}/.jackin/config.toml", w.home))
                ),
            ),
            Prop::new("Changes", plural(n, "change", "changes")),
        ];
        if self.general_dirty() {
            facts.push(Prop::new(
                "Commits",
                format!(
                    "co-author trailer {} · DCO sign-off {}",
                    on_off(self.pending.coauthor_trailer),
                    on_off(self.pending.dco_signoff)
                ),
            ));
        }
        facts.extend(self.cfg.summary_facts());
        if self.trust_dirty() {
            let changed: Vec<String> = self
                .pending
                .trust
                .iter()
                .zip(&self.original.trust)
                .filter(|(a, b)| a.trusted != b.trusted)
                .map(|(a, _)| {
                    format!(
                        "{} → {}",
                        a.source,
                        if a.trusted { "trusted" } else { "untrusted" }
                    )
                })
                .collect();
            facts.push(Prop::new("Trust", changed.join(" · ")));
        }
        for s in &sensitive {
            facts.push(
                Prop::new(
                    "Sensitive",
                    format!("{s} · sensitive global mount path detected"),
                )
                .tone(Tone::Warning),
            );
        }
        for b in &blockers {
            facts.push(Prop::new("Blocker", b.clone()).tone(Tone::Error));
        }
        let mut code = self.cfg.diff_lines();
        if self.pending.coauthor_trailer != self.original.coauthor_trailer {
            code.push(format!(
                "~ coauthor_trailer {} → {}",
                self.original.coauthor_trailer, self.pending.coauthor_trailer
            ));
        }
        if self.pending.dco_signoff != self.original.dco_signoff {
            code.push(format!(
                "~ dco_signoff {} → {}",
                self.original.dco_signoff, self.pending.dco_signoff
            ));
        }
        for (a, b) in self.pending.trust.iter().zip(&self.original.trust) {
            if a.trusted != b.trusted {
                code.push(format!(
                    "~ trust {} {} → {}",
                    a.source, b.trusted, a.trusted
                ));
            }
        }
        let (confirm, tag) = if !blockers.is_empty() {
            (
                Button::primary(WidgetId::of("settings.preview.cleanup"), "Clean up & save…"),
                "preview.cleanup",
            )
        } else if !sensitive.is_empty() {
            (
                Button::primary(WidgetId::of("settings.preview.save"), "Save anyway"),
                "preview",
            )
        } else {
            (
                Button::primary(WidgetId::of("settings.preview.save"), "Save"),
                "preview",
            )
        };
        let d = Dialog::facts(
            WidgetId::of("settings.preview"),
            "Save settings",
            facts,
            code,
            None,
            confirm,
        );
        cx.open(Modal::Dialog(d), ModalTag::new(tag));
    }

    fn start_save(&mut self, w: &mut World, cx: &mut Cx) {
        self.sync_pending();
        self.saving = true;
        self.save.busy = true;
        let ok = !w.save_fails_once;
        w.save_fails_once = false;
        w.schedule(900, Msg::GlobalSaved { ok });
        cx.status("Saving settings…");
        cx.focus.focus(SAVE);
    }

    fn finish_save(&mut self, ok: bool, w: &mut World, cx: &mut Cx) {
        self.saving = false;
        self.save.busy = false;
        if !ok {
            let d = InfoDialog::new(
                WidgetId::of("settings.savefail"),
                "Settings error",
                vec![
                    Prop::new(
                        "Error",
                        "write failed: ~/.jackin/config.toml is not writable (EACCES)",
                    ),
                    Prop::new("State", "your edits are intact · nothing was written"),
                    Prop::new("Next", "fix the permission and Save again"),
                ],
            )
            .error()
            .width(66);
            cx.open(Modal::Info(d), ModalTag::new("savefail"));
            return;
        }
        w.global = self.pending.clone();
        for t in &self.pending.trust {
            for r in w.roles.iter_mut() {
                if r.source.label().starts_with(&t.source)
                    || t.source.trim_start_matches("~/").is_empty()
                {
                    continue;
                }
                if r.source.label().contains(&t.source) {
                    r.trusted = t.trusted;
                }
            }
        }
        self.original = self.pending.clone();
        cx.status("Settings saved");
        cx.go(Go::Manager);
    }

    fn leave(&mut self, cx: &mut Cx) {
        self.sync_pending();
        if self.change_count() > 0 {
            let d = Dialog::confirm(
                WidgetId::of("settings.exit"),
                "Unsaved changes",
                &format!(
                    "Save changes before leaving? {} would be lost.",
                    plural(self.change_count(), "change", "changes")
                ),
                "Save",
            )
            .with_actions(
                vec![
                    Button::subtle(WidgetId::of("settings.exit.cancel"), "Cancel"),
                    Button::danger(WidgetId::of("settings.exit.discard"), "Discard"),
                    Button::primary(WidgetId::of("settings.exit.save"), "Save"),
                ],
                Some(0),
            );
            cx.open(Modal::Dialog(d), ModalTag::new("exit"));
        } else {
            cx.go(Go::Manager);
        }
    }

    fn render_general(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let bg = t.canvas;
        let x = area.x + 2;
        let mut y = area.y;
        buf.set_string(
            x,
            y,
            "Commits",
            t.secondary().bg(bg).add_modifier(Modifier::BOLD),
        );
        y += 1;
        self.coauthor.render(Rect::new(x, y, 40, 1), buf, ctx, bg);
        if self.pending.coauthor_trailer != self.original.coauthor_trailer {
            buf.set_string(
                x + 41,
                y,
                "•",
                ratatui::style::Style::new().fg(t.warning).bg(bg),
            );
        }
        y += 1;
        self.dco.render(Rect::new(x, y, 40, 1), buf, ctx, bg);
        if self.pending.dco_signoff != self.original.dco_signoff {
            buf.set_string(
                x + 41,
                y,
                "•",
                ratatui::style::Style::new().fg(t.warning).bg(bg),
            );
        }
        y += 2;
        buf.set_string(
            x,
            y,
            "Two independent flags; both apply to every Workspace.",
            t.faint().bg(bg),
        );
        y += 2;
        buf.set_string(
            x,
            y,
            "Trailers",
            t.secondary().bg(bg).add_modifier(Modifier::BOLD),
        );
        y += 1;
        buf.set_string(x, y, truncate("Co-authored-by: <agent> <noreply@…>   ·   Signed-off-by: Alexey Zhokhov <alexey@chainargos.com>", area.width.saturating_sub(4) as usize), t.muted().bg(bg));
    }

    fn render_trust(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(TRUST);
        let trusted = self.pending.trust.iter().filter(|r| r.trusted).count();
        let untrusted = self.pending.trust.len() - trusted;
        buf.set_string(
            area.x + 2,
            area.y,
            "Role sources",
            t.secondary().bg(bg).add_modifier(Modifier::BOLD),
        );
        let meta = format!(
            "{} · {}",
            plural(trusted, "trusted", "trusted"),
            plural(untrusted, "untrusted", "untrusted")
        );
        buf.set_string(
            area.right().saturating_sub(width(&meta) as u16 + 2),
            area.y,
            &meta,
            t.faint().bg(bg),
        );
        let body = Rect::new(
            area.x,
            area.y + 1,
            area.width,
            area.height.saturating_sub(1),
        );
        ctx.control(TRUST, body, false);
        let src_w = area.width.saturating_sub(30).clamp(16, 40) as usize;
        for (i, row) in self.pending.trust.iter().enumerate() {
            let y = body.y + i as u16;
            if y >= body.bottom() {
                break;
            }
            let rid = TRUST.child(i);
            let mut s = ctx.state(rid);
            s.focused = focused && i == self.trust_cursor;
            s.selected = i == self.trust_cursor;
            let st = t.row(s, bg);
            let rect = Rect::new(body.x, y, body.width, 1);
            fill(buf, rect, st);
            buf.set_string(rect.x, y, "▎", t.gutter(s, st.bg.unwrap_or(bg), false));
            buf.set_string(
                rect.x + 1,
                y,
                if s.selected { "›" } else { " " },
                st.fg(if s.focused {
                    t.accent
                } else {
                    t.text_secondary
                }),
            );
            let changed = self
                .original
                .trust
                .get(i)
                .is_some_and(|o| o.trusted != row.trusted);
            buf.set_string(
                rect.x + 3,
                y,
                if changed { "•" } else { " " },
                st.fg(t.warning),
            );
            buf.set_string(
                rect.x + 5,
                y,
                fit(&truncate(&row.source, src_w), src_w),
                if s.selected {
                    st.add_modifier(Modifier::BOLD)
                } else {
                    st
                },
            );
            let x = rect.x + 7 + src_w as u16;
            buf.set_string(x, y, fit(row.kind, 5), st.fg(t.text_muted));
            let mark = if row.trusted {
                "[✓] trusted"
            } else {
                "[ ] untrusted"
            };
            buf.set_string(
                x + 6,
                y,
                mark,
                st.fg(if row.trusted {
                    t.text_primary
                } else {
                    t.warning
                }),
            );
            let roles = plural(row.roles, "role", "roles");
            if x + 22 + (width(&roles) as u16) < rect.right() {
                buf.set_string(x + 21, y, &roles, st.fg(t.text_faint));
            }
            ctx.clickable(rid, rect);
        }
        let ny = body.y + self.pending.trust.len() as u16 + 1;
        if ny + 1 < body.bottom() {
            buf.set_string(
                area.x + 2,
                ny,
                truncate(
                    "An untrusted source blocks + Load role until it is trusted here or",
                    area.width.saturating_sub(4) as usize,
                ),
                t.faint().bg(bg),
            );
            buf.set_string(
                area.x + 2,
                ny + 1,
                truncate(
                    "in the trust dialog that the load opens.",
                    area.width.saturating_sub(4) as usize,
                ),
                t.faint().bg(bg),
            );
        }
    }

    fn trust_key(&mut self, key: &Key, cx: &mut Cx) -> Outcome {
        let n = self.pending.trust.len();
        if n == 0 {
            return Outcome::Ignored;
        }
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.trust_cursor = self.trust_cursor.saturating_sub(1)
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.trust_cursor = (self.trust_cursor + 1).min(n - 1)
            }
            KeyCode::Home | KeyCode::Char('g') => self.trust_cursor = 0,
            KeyCode::End | KeyCode::Char('G') => self.trust_cursor = n - 1,
            KeyCode::Char(' ') | KeyCode::Enter => {
                let r = &mut self.pending.trust[self.trust_cursor];
                r.trusted = !r.trusted;
                cx.status(format!(
                    "{} · {} · save to apply",
                    r.source,
                    if r.trusted { "trusted" } else { "untrusted" }
                ));
            }
            KeyCode::Char('o') => {
                let r = &self.pending.trust[self.trust_cursor];
                if r.kind == "git" && r.source.contains("github.com") {
                    cx.status(format!("Opened https://{} in the browser", r.source));
                } else {
                    cx.status(format!("Revealed {} in Finder", r.source));
                }
            }
            _ => return Outcome::Ignored,
        }
        Outcome::Changed
    }
}

fn on_off(b: bool) -> &'static str {
    if b { "on" } else { "off" }
}

fn is_sensitive(path: &str) -> bool {
    let p = path.trim_start_matches('~');
    p == "/"
        || p.starts_with("/.ssh")
        || p.starts_with("/.aws")
        || p.starts_with("/.gnupg")
        || p.starts_with("/etc")
        || p.is_empty()
        || p.starts_with("/.config/op")
}

impl Screen for SettingsScreen {
    fn enter(&mut self, _w: &mut World, cx: &mut Cx) {
        cx.focus.focus(TABS);
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(TABS)
    }

    fn animating(&self, _w: &World) -> bool {
        self.saving
    }

    fn on_tick(&mut self, _w: &mut World, _cx: &mut Cx) -> Outcome {
        if self.saving {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn on_msg(&mut self, msg: &Msg, w: &mut World, cx: &mut Cx) -> Outcome {
        if let Msg::GlobalSaved { ok } = msg
            && self.saving
        {
            self.finish_save(*ok, w, cx);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.saving {
            return Outcome::Consumed;
        }
        let focus = cx.focus.current();
        if key.ctrl() && key.code == KeyCode::Char('s') {
            self.open_preview(w, cx);
            return Outcome::Changed;
        }
        match key.code {
            KeyCode::Char('[') => {
                let i = (self.tabs.active + 4) % 5;
                self.set_tab(i, cx);
                return Outcome::Changed;
            }
            KeyCode::Char(']') => {
                let i = (self.tabs.active + 1) % 5;
                self.set_tab(i, cx);
                return Outcome::Changed;
            }
            _ => {}
        }
        if focus == Some(TABS) {
            if matches!(
                key.code,
                KeyCode::Enter | KeyCode::Down | KeyCode::Char('j')
            ) {
                cx.focus.focus(self.body_focus());
                return Outcome::Changed;
            }
            let (o, ev) = self.tabs.on_key(key);
            if let Some(TabEvent::Activated(i)) = ev {
                self.tab = StTab::ALL[i.min(4)];
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
            match key.code {
                KeyCode::Esc => {
                    self.leave(cx);
                    return Outcome::Changed;
                }
                _ => return Outcome::Ignored,
            }
        }
        if focus == Some(CANCEL) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                self.leave(cx);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if focus == Some(SAVE) {
            let (o, fired) = self.save.on_key(key);
            if fired {
                self.open_preview(w, cx);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if matches!(focus, Some(CANCEL) | Some(SAVE)) {
            match key.code {
                KeyCode::Left | KeyCode::Char('h') => {
                    cx.focus.focus(CANCEL);
                    return Outcome::Changed;
                }
                KeyCode::Right | KeyCode::Char('l') => {
                    cx.focus.focus(SAVE);
                    return Outcome::Changed;
                }
                KeyCode::Esc | KeyCode::Up | KeyCode::Char('k') => {
                    cx.focus.focus(self.body_focus());
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        let body = match self.tab {
            StTab::General if focus == Some(COAUTHOR) => {
                let o = self.coauthor.on_key(key);
                if o.consumed() {
                    self.sync_pending();
                    o
                } else {
                    match key.code {
                        KeyCode::Down | KeyCode::Char('j') => {
                            cx.focus.focus(DCO);
                            Outcome::Changed
                        }
                        _ => Outcome::Ignored,
                    }
                }
            }
            StTab::General if focus == Some(DCO) => {
                let o = self.dco.on_key(key);
                if o.consumed() {
                    self.sync_pending();
                    o
                } else {
                    match key.code {
                        KeyCode::Up | KeyCode::Char('k') => {
                            cx.focus.focus(COAUTHOR);
                            Outcome::Changed
                        }
                        _ => Outcome::Ignored,
                    }
                }
            }
            StTab::Trust if focus == Some(TRUST) => self.trust_key(key, cx),
            t if t.cfg().is_some() && focus == Some(self.cfg.list_id(t.cfg().unwrap())) => {
                let o = self.cfg.on_key(t.cfg().unwrap(), key, w, cx);
                if o.consumed() {
                    self.sync_pending();
                }
                o
            }
            _ => Outcome::Ignored,
        };
        if body.consumed() {
            return body;
        }
        match key.code {
            KeyCode::Esc => {
                cx.focus.focus(TABS);
                Outcome::Changed
            }
            KeyCode::Char('c') if key.plain() && self.tab == StTab::Auth => {
                cx.go(Go::Accounts { select: None });
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        if self.saving {
            return Outcome::Consumed;
        }
        if self.tabs.owns(id) {
            let (o, ev) = self.tabs.on_click(id);
            if let Some(TabEvent::Activated(i)) = ev {
                self.tab = StTab::ALL[i.min(4)];
            }
            cx.focus.focus(TABS);
            return o.or(Outcome::Changed);
        }
        if id == CANCEL {
            cx.focus.focus(CANCEL);
            if self.cancel.on_click() {
                self.leave(cx);
            }
            return Outcome::Changed;
        }
        if id == SAVE {
            cx.focus.focus(SAVE);
            if self.save.on_click() {
                self.open_preview(w, cx);
            }
            return Outcome::Changed;
        }
        match self.tab {
            StTab::General => {
                if id == COAUTHOR {
                    cx.focus.focus(COAUTHOR);
                    let o = self.coauthor.on_click();
                    self.sync_pending();
                    return o;
                }
                if id == DCO {
                    cx.focus.focus(DCO);
                    let o = self.dco.on_click();
                    self.sync_pending();
                    return o;
                }
                Outcome::Ignored
            }
            StTab::Trust => {
                for i in 0..self.pending.trust.len() {
                    if TRUST.child(i) == id {
                        let same = self.trust_cursor == i;
                        self.trust_cursor = i;
                        cx.focus.focus(TRUST);
                        if same {
                            let k = Key {
                                code: KeyCode::Char(' '),
                                mods: ratatui::crossterm::event::KeyModifiers::NONE,
                            };
                            return self.trust_key(&k, cx);
                        }
                        return Outcome::Changed;
                    }
                }
                Outcome::Ignored
            }
            t => {
                let o = self.cfg.on_click(t.cfg().unwrap(), id, pos, w, cx);
                if o.consumed() {
                    self.sync_pending();
                }
                o
            }
        }
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        match self.tab.cfg() {
            Some(t) => self.cfg.on_wheel(t, id, delta),
            None => Outcome::Ignored,
        }
    }

    fn form_changed(&mut self, tag: &ModalTag, form: &mut super::modals::FormDialog, w: &World) {
        self.cfg.form_changed(tag, form, w);
    }

    fn on_modal(
        &mut self,
        tag: &ModalTag,
        result: ModalResult,
        w: &mut World,
        cx: &mut Cx,
    ) -> Outcome {
        if let Some(o) = self.cfg.on_modal(tag, result.clone(), w, cx) {
            self.sync_pending();
            return o;
        }
        match (tag.kind, result) {
            (
                "preview",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                self.start_save(w, cx);
                Outcome::Changed
            }
            (
                "preview.cleanup",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                self.cfg.clean_up();
                cx.status("Cleaned up isolated state · saving");
                self.start_save(w, cx);
                Outcome::Changed
            }
            ("preview" | "preview.cleanup", _) => {
                cx.status("Save aborted · settings unchanged");
                cx.focus.focus(SAVE);
                Outcome::Changed
            }
            (
                "exit",
                ModalResult::Dialog {
                    action: Some(2), ..
                },
            ) => {
                self.open_preview(w, cx);
                Outcome::Changed
            }
            (
                "exit",
                ModalResult::Dialog {
                    action: Some(1), ..
                },
            ) => {
                cx.status(format!(
                    "Discarded {}",
                    plural(self.change_count(), "change", "changes")
                ));
                cx.go(Go::Manager);
                Outcome::Changed
            }
            ("savefail", _) => {
                cx.focus.focus(SAVE);
                Outcome::Changed
            }
            _ => Outcome::Changed,
        }
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        self.sync_pending();
        let items: Vec<TabItem> = StTab::ALL
            .iter()
            .zip(TAB_NAMES)
            .map(|(tab, name)| {
                let mut it = TabItem::new(name);
                it.dirty = match tab {
                    StTab::General => self.general_dirty(),
                    StTab::Trust => self.trust_dirty(),
                    t => self.cfg.tab_dirty(t.cfg().unwrap()),
                };
                it.error = match tab {
                    StTab::General | StTab::Trust => false,
                    t => self.cfg.tab_error(t.cfg().unwrap()),
                };
                it
            })
            .collect();
        let active = self.tabs.active;
        self.tabs = Tabs::with_items(TABS, items);
        self.tabs.set_active(active);
        self.tabs.quiet = false;
        self.tabs.render(
            Rect::new(area.x + 1, area.y + 1, area.width.saturating_sub(2), 2),
            buf,
            ctx,
            bg,
        );
        let body = Rect::new(
            area.x + 1,
            area.y + 4,
            area.width.saturating_sub(2),
            area.height.saturating_sub(6),
        );
        self.row_status = None;
        match self.tab {
            StTab::General => self.render_general(body, buf, ctx),
            StTab::Trust => self.render_trust(body, buf, ctx),
            t => {
                self.row_status = self.cfg.render(t.cfg().unwrap(), body, buf, ctx, w);
            }
        }
        let n = self.change_count();
        self.save.disabled = n == 0 && !self.saving;
        self.save.label = if self.saving {
            format!("{} Saving…", spinner_frame(w.now_ms() as u64 / 80))
        } else {
            "Save…".into()
        };
        let widths = [self.cancel.width(), self.save.width()];
        let rects = row_layout_right(
            Rect::new(
                area.x,
                area.bottom().saturating_sub(1),
                area.width.saturating_sub(4),
                1,
            ),
            &widths,
            3,
        );
        self.cancel.render(rects[0], buf, ctx, bg);
        self.save.render(rects[1], buf, ctx, bg);
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if self.saving {
            return vec![hint("", "Saving…")];
        }
        let mut v = vec![];
        if focus == Some(TABS) {
            v.push(hint("← →", "Tab"));
            v.push(hint("1–5", "Jump"));
            v.push(hint("Enter", "Body"));
        } else if matches!(focus, Some(CANCEL) | Some(SAVE)) {
            v.push(hint("← →", "Choose"));
            v.push(hint("Enter", "Run"));
        } else {
            match self.tab {
                StTab::General => {
                    v.push(hint("Space", "Toggle"));
                    v.push(hint("↑↓", "Move"));
                }
                StTab::Trust => {
                    v.push(hint("Space", "Toggle trust"));
                    v.push(hint("o", "Open source"));
                }
                t => v.extend(self.cfg.hints(t.cfg().unwrap())),
            }
        }
        v.push(hint("[ ]", "Switch tab"));
        v.push(hint("Ctrl+S", "Save"));
        v.push(hint("Esc", "Back"));
        v
    }

    fn crumb(&self, _w: &World) -> String {
        format!("Settings › global › {}", TAB_NAMES[self.tabs.active.min(4)])
    }

    fn strip_right(&self, w: &World) -> Vec<Segment> {
        let mut v = vec![];
        if self.saving {
            v.push(
                Segment::new(
                    format!("{} saving…", spinner_frame(w.now_ms() as u64 / 80)),
                    Tone::Secondary,
                )
                .priority(9),
            );
        }
        let n = self.change_count();
        if n > 0 {
            v.push(
                Segment::new(
                    format!("• {}", plural(n, "change", "changes")),
                    Tone::Warning,
                )
                .priority(8),
            );
        }
        if let Some(s) = &self.row_status {
            v.push(Segment::new(s.clone(), Tone::Muted).priority(3));
        }
        v
    }

    fn on_esc_top(&mut self, _w: &mut World, cx: &mut Cx) -> Outcome {
        if cx.focus.is(TABS) {
            self.leave(cx);
        } else {
            cx.focus.focus(TABS);
        }
        Outcome::Changed
    }
}
