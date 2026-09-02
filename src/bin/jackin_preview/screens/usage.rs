//! Host Usage: a read-only projection of every provider account with an
//! honest overview on row zero. Nothing here mutates; `m` hands the
//! selected row to the Account & Usage Center.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::id::WidgetId;
use junie_tui::core::scroll::ScrollState;
use junie_tui::theme::Tone;
use junie_tui::ui::ctx::{RenderCtx, fill};
use junie_tui::ui::text::{fit, truncate, width};
use junie_tui::widgets::empty::{self, EmptyState};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::progress::{Meter, MeterTone, MeterVisual, spinner_frame};
use junie_tui::widgets::scrollbar;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};
use ratatui::style::{Modifier, Style};

use super::{Cx, Go, Screen, plural};
use crate::domain::account::{Account, AccountId, IssueCode, Lifecycle};
use crate::domain::agent::UsageSurface;
use crate::domain::usage::{Freshness, OverallSummary, QuotaStatus};
use crate::sim::provider;
use crate::sim::world::{Msg, World};

pub const LIST: WidgetId = WidgetId::of("usage.list");
pub const DETAIL: WidgetId = WidgetId::of("usage.detail");

#[derive(Debug, Clone, PartialEq, Eq)]
enum Row {
    Overview,
    Heading(UsageSurface),
    Account(AccountId),
}

/// One detail row: text, tone and an optional meter (used %, tone).
type DetailLine = (String, Tone, Option<(u8, MeterTone)>);

#[derive(Default)]
pub struct UsageScreen {
    pub selected: Option<AccountId>,
    rows: Vec<Row>,
    scroll: ScrollState,
    detail_scroll: ScrollState,
    list_area: Rect,
    detail_area: Rect,
    /// Narrow terminals swap the list for the detail.
    pub show_detail: bool,
    pub refreshing: bool,
}

impl UsageScreen {
    pub fn select(&mut self, id: Option<AccountId>) {
        self.selected = id;
    }

    fn build_rows(&mut self, w: &World) {
        let mut rows = vec![Row::Overview];
        for s in UsageSurface::ALL {
            let accounts: Vec<&Account> = w
                .accounts
                .sorted()
                .into_iter()
                .filter(|a| a.surface == s)
                .collect();
            if accounts.is_empty() && s != UsageSurface::Unsupported {
                continue;
            }
            rows.push(Row::Heading(s));
            for a in accounts {
                rows.push(Row::Account(a.id.clone()));
            }
        }
        self.rows = rows;
        if let Some(id) = &self.selected
            && !self
                .rows
                .iter()
                .any(|r| matches!(r, Row::Account(x) if x == id))
        {
            self.selected = None;
        }
        self.scroll.set_content(self.rows.len());
    }

    fn cursor(&self) -> usize {
        match &self.selected {
            Some(id) => self
                .rows
                .iter()
                .position(|r| matches!(r, Row::Account(x) if x == id))
                .unwrap_or(0),
            None => 0,
        }
    }

    fn move_cursor(&mut self, delta: isize) {
        let n = self.rows.len();
        if n == 0 {
            return;
        }
        let mut i = self.cursor() as isize;
        loop {
            i = (i + delta).clamp(0, n as isize - 1);
            match &self.rows[i as usize] {
                Row::Heading(_) => {
                    if i == 0 || i == n as isize - 1 {
                        break;
                    }
                    continue;
                }
                Row::Overview => {
                    self.selected = None;
                    break;
                }
                Row::Account(id) => {
                    self.selected = Some(id.clone());
                    break;
                }
            }
        }
        self.scroll.ensure_visible(self.cursor());
        self.detail_scroll.jump_start();
    }

    fn refresh(&mut self, w: &mut World, cx: &mut Cx) {
        let mut n = 0;
        let ids: Vec<AccountId> = w
            .accounts
            .accounts
            .iter()
            .filter(|a| a.enabled)
            .map(|a| a.id.clone())
            .collect();
        for (i, id) in ids.iter().enumerate() {
            if let Some(a) = w.accounts.get_mut(id)
                && a.usage.freshness.phase != Freshness::Refreshing
            {
                a.usage.freshness.phase = Freshness::Refreshing;
                let d = provider::refresh_duration_ms(a) + i as i64 * 120;
                w.schedule(
                    d,
                    Msg::AccountRefreshed {
                        account: id.clone(),
                    },
                );
                n += 1;
            }
        }
        if n == 0 {
            cx.status("Nothing to refresh");
        } else {
            self.refreshing = true;
            cx.status(format!(
                "Reloading the broker projection · {}",
                plural(n, "account", "accounts")
            ));
        }
    }

    fn account_lines(a: &Account, w: &World) -> Vec<DetailLine> {
        let mut v: Vec<DetailLine> = vec![];
        v.push((
            format!(
                "Provider     {} · surface {}",
                a.provider.label(),
                a.surface.surface_name()
            ),
            Tone::Normal,
            None,
        ));
        v.push((
            format!(
                "Account      {} · {}",
                a.identity.label(),
                a.identity.plan.clone().unwrap_or("plan unknown".into())
            ),
            if a.identity.subject.is_some() {
                Tone::Normal
            } else {
                Tone::Muted
            },
            None,
        ));
        v.push((
            format!(
                "Credential   {} · {}",
                a.source.origin_label(),
                a.source.safe_detail()
            ),
            Tone::Normal,
            None,
        ));
        let status = match a.usage.freshness.phase {
            Freshness::Current => format!(
                "{} · current · refreshed {}",
                a.lifecycle.label(),
                a.last_refresh_secs
                    .map(|s| w.clock.ago(s))
                    .unwrap_or("never".into())
            ),
            Freshness::Stale => format!(
                "{} · stale · last good {}",
                a.lifecycle.label(),
                a.usage
                    .freshness
                    .last_good_secs
                    .map(|s| w.clock.ago(s))
                    .unwrap_or("?".into())
            ),
            Freshness::Refreshing => format!("{} · refreshing…", a.lifecycle.label()),
            Freshness::Failed => format!(
                "{} · error: {}",
                a.lifecycle.label(),
                a.issue
                    .as_ref()
                    .map(|i| i.message.clone())
                    .unwrap_or("refresh failed".into())
            ),
        };
        v.push((
            format!("Status       {status}"),
            match a.usage.freshness.phase {
                Freshness::Failed => Tone::Error,
                Freshness::Stale => Tone::Warning,
                _ => Tone::Normal,
            },
            None,
        ));
        v.push((String::new(), Tone::Normal, None));
        v.push(("Limits".into(), Tone::Secondary, None));
        if a.usage.windows.is_empty() {
            v.push((
                match a.lifecycle {
                    Lifecycle::NeedsLogin => {
                        "  needs login · no quota until the agent signs in".into()
                    }
                    Lifecycle::NeedsSecret => {
                        "  needs secret · no quota until a key is present".into()
                    }
                    Lifecycle::Unsupported => {
                        "  unsupported · this provider exposes no usage".into()
                    }
                    _ => "  not started".into(),
                },
                Tone::Muted,
                None,
            ));
        }
        for win in &a.usage.windows {
            if win.has_meter() {
                let tone = match a.usage.freshness.phase {
                    Freshness::Refreshing => MeterTone::Refreshing,
                    Freshness::Stale | Freshness::Failed => MeterTone::Stale,
                    Freshness::Current => match win.status {
                        QuotaStatus::Exhausted => MeterTone::Exhausted,
                        QuotaStatus::Warning => MeterTone::Warning,
                        _ => MeterTone::Normal,
                    },
                };
                let mut text = format!("  {}", win.label);
                let rest = super::accounts::meter_detail(win, w);
                text.push('\u{1}');
                text.push_str(&rest);
                v.push((text, Tone::Normal, Some((win.used_pct.unwrap_or(0), tone))));
            } else {
                let value = win.value_label();
                let text = if value.to_lowercase().starts_with(&win.label.to_lowercase()) {
                    format!("  {value}")
                } else {
                    format!("  {}   {value}", win.label)
                };
                v.push((
                    text,
                    if win.status == QuotaStatus::Error {
                        Tone::Error
                    } else {
                        Tone::Muted
                    },
                    None,
                ));
            }
        }
        if a.usage.freshness.phase != Freshness::Current && !a.usage.windows.is_empty() {
            v.push((String::new(), Tone::Normal, None));
            v.push((
                format!(
                    "Last good    kept from {}{}",
                    a.usage
                        .freshness
                        .last_good_secs
                        .map(|s| w.clock.ago(s))
                        .unwrap_or("?".into()),
                    a.issue
                        .as_ref()
                        .filter(|i| i.code == IssueCode::QuotaUnsupported)
                        .map(|_| " · quota not visible")
                        .unwrap_or("")
                ),
                Tone::Muted,
                None,
            ));
        }
        v
    }

    fn overview_lines(w: &World) -> Vec<(String, Tone)> {
        let s = OverallSummary::compute(&w.accounts.accounts);
        let mut v = vec![
            (
                format!("Health       {} · {}", s.health.label(), s.issues_line()),
                match s.health {
                    crate::domain::usage::HealthWord::Degraded
                    | crate::domain::usage::HealthWord::Blocked => Tone::Error,
                    crate::domain::usage::HealthWord::Attention => Tone::Warning,
                    _ => Tone::Normal,
                },
            ),
            (
                format!(
                    "Freshness    {} of {} current · broker projection {}",
                    s.counts.enabled - s.counts.stale - s.counts.failed - s.counts.refreshing,
                    s.counts.enabled,
                    w.clock.ago(w.last_refresh_secs)
                ),
                Tone::Normal,
            ),
            (format!("Registry     {}", s.counts_line()), Tone::Normal),
            (String::new(), Tone::Normal),
            (
                format!(
                    "{:<12} {:<9} {:<26} {}",
                    "Provider", "Accounts", "Worst window", "Status"
                ),
                Tone::Muted,
            ),
        ];
        for surface in UsageSurface::ALL {
            let accounts: Vec<&Account> = w
                .accounts
                .accounts
                .iter()
                .filter(|a| a.surface == surface && a.enabled)
                .collect();
            if surface == UsageSurface::Unsupported {
                v.push((
                    format!(
                        "{:<12} {:<9} {:<26} {}",
                        "—", "—", "—", "unsupported sentinel"
                    ),
                    Tone::Muted,
                ));
                continue;
            }
            if accounts.is_empty() {
                v.push((
                    format!(
                        "{:<12} {:<9} {:<26} {}",
                        surface.label(),
                        "—",
                        "—",
                        "not discovered"
                    ),
                    Tone::Faint,
                ));
                continue;
            }
            let worst = accounts
                .iter()
                .flat_map(|a| a.usage.windows.iter().map(move |w| (a, w)))
                .filter(|(_, w)| w.used_pct.is_some())
                .max_by_key(|(_, w)| w.used_pct.unwrap_or(0));
            let (win_text, status, tone) = match worst {
                Some((a, win)) => {
                    let st = match (win.status, a.usage.freshness.phase) {
                        (QuotaStatus::Exhausted, _) => ("! exhausted".to_owned(), Tone::Error),
                        (_, Freshness::Failed) => (
                            format!(
                                "! {}",
                                a.issue
                                    .as_ref()
                                    .map(|i| i
                                        .message
                                        .split(':')
                                        .next()
                                        .unwrap_or("error")
                                        .to_lowercase())
                                    .unwrap_or("error".into())
                            ),
                            Tone::Error,
                        ),
                        (QuotaStatus::Warning, _) => ("▲ warning".to_owned(), Tone::Warning),
                        (_, Freshness::Stale) => ("▲ stale".to_owned(), Tone::Warning),
                        _ => ("current".to_owned(), Tone::Normal),
                    };
                    (
                        format!("{} {}%", win.label, win.used_pct.unwrap_or(0)),
                        st.0,
                        st.1,
                    )
                }
                None => {
                    let a = accounts[0];
                    ("—".to_owned(), a.status_word().to_owned(), Tone::Muted)
                }
            };
            v.push((
                format!(
                    "{:<12} {:<9} {:<26} {}",
                    surface.label(),
                    accounts.len(),
                    truncate(&win_text, 26),
                    status
                ),
                tone,
            ));
        }
        let unresolved: Vec<&Account> = w
            .accounts
            .accounts
            .iter()
            .filter(|a| a.enabled && a.identity.subject.is_none())
            .collect();
        if !unresolved.is_empty() {
            v.push((String::new(), Tone::Normal));
            for a in unresolved {
                v.push((
                    format!(
                        "Unresolved   {} · {} ({}) — not an authenticated account",
                        a.surface.label(),
                        a.source.safe_detail(),
                        a.confidence.label()
                    ),
                    Tone::Muted,
                ));
            }
        }
        v.push((String::new(), Tone::Normal));
        v.push(("Rollups sum identical windows and units only; a provider with mixed windows shows its worst window.".into(), Tone::Faint));
        if !s.comparable.is_empty() {
            v.push((String::new(), Tone::Normal));
            for c in &s.comparable {
                v.push((
                    format!(
                        "Comparable   {} · {} · {} · {}–{}% remaining",
                        c.surface.label(),
                        c.label,
                        plural(c.accounts, "account", "accounts"),
                        c.min_remaining_pct,
                        c.max_remaining_pct
                    ),
                    Tone::Secondary,
                ));
            }
        }
        v
    }

    fn draw_list(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(LIST);
        self.list_area = area;
        self.scroll.set_viewport(area.height as usize);
        self.scroll.ensure_visible(self.cursor());
        ctx.control(LIST, area, false);
        ctx.scrollable(LIST, area);
        let has_sb = self.scroll.overflows();
        let row_w = area.width.saturating_sub(u16::from(has_sb));
        let cur = self.cursor();
        for (k, i) in self.scroll.visible_range().enumerate() {
            let y = area.y + k as u16;
            let rid = LIST.child(i);
            match &self.rows[i] {
                Row::Heading(s) => {
                    buf.set_string(area.x + 2, y, s.label(), t.faint().bg(bg));
                }
                Row::Overview | Row::Account(_) => {
                    let mut st8 = ctx.state(rid);
                    st8.focused = focused && i == cur;
                    st8.selected = i == cur;
                    let st = t.row(st8, bg);
                    let r = Rect::new(area.x, y, row_w, 1);
                    fill(buf, r, st);
                    buf.set_string(r.x, y, "▎", t.gutter(st8, st.bg.unwrap_or(bg), false));
                    if st8.selected {
                        buf.set_string(
                            r.x + 1,
                            y,
                            "›",
                            st.fg(if focused { t.accent } else { t.text_secondary }),
                        );
                    }
                    match &self.rows[i] {
                        Row::Overview => buf.set_string(r.x + 3, y, "Overview", st),
                        Row::Account(id) => {
                            let Some(a) = w.accounts.get(id) else {
                                continue;
                            };
                            let mut label = a.display_name.clone();
                            if a.default_for_provider {
                                label.push_str("  ★");
                            }
                            let health = if !a.enabled {
                                ""
                            } else if a.is_error_state()
                                || matches!(a.usage.worst_status(), Some(QuotaStatus::Exhausted))
                            {
                                "!"
                            } else if matches!(a.usage.worst_status(), Some(QuotaStatus::Warning))
                                || a.usage.freshness.phase == Freshness::Stale
                            {
                                "▲"
                            } else {
                                ""
                            };
                            let meta = if a.usage.freshness.phase == Freshness::Refreshing {
                                spinner_frame(w.now_ms() as u64 / 80).to_owned()
                            } else if !a.enabled {
                                "disabled".into()
                            } else if a.lifecycle != Lifecycle::Available {
                                a.lifecycle.label().to_owned()
                            } else {
                                String::new()
                            };
                            let mw = width(&meta) as u16 + if health.is_empty() { 0 } else { 2 };
                            let lw = r.width.saturating_sub(5 + mw + 2) as usize;
                            buf.set_string(
                                r.x + 5,
                                y,
                                fit(&truncate(&label, lw), lw),
                                if a.enabled { st } else { st.fg(t.text_faint) },
                            );
                            let mut rx = r.right().saturating_sub(1);
                            if !meta.is_empty() {
                                rx = rx.saturating_sub(width(&meta) as u16);
                                buf.set_string(
                                    rx,
                                    y,
                                    &meta,
                                    st.fg(t.text_muted).remove_modifier(Modifier::BOLD),
                                );
                                rx = rx.saturating_sub(1);
                            }
                            if !health.is_empty() {
                                rx = rx.saturating_sub(1);
                                buf.set_string(
                                    rx,
                                    y,
                                    health,
                                    st.fg(if health == "!" { t.error } else { t.warning }),
                                );
                            }
                        }
                        _ => {}
                    }
                    ctx.clickable(rid, r);
                }
            }
        }
        if has_sb {
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

    fn draw_detail(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        let t = ctx.theme;
        let bg = t.canvas;
        let focused = ctx.interaction.focused(DETAIL);
        self.detail_area = area;
        ctx.control(DETAIL, area, false);
        ctx.scrollable(DETAIL, area);
        let title_row = area.y;
        let body = Rect::new(
            area.x,
            area.y + 2,
            area.width,
            area.height.saturating_sub(2),
        );
        let meter_w = if body.width >= 70 { 24 } else { 16 };
        match self
            .selected
            .clone()
            .and_then(|id| w.accounts.get(&id).cloned())
        {
            None => {
                let s = OverallSummary::compute(&w.accounts.accounts);
                buf.set_string(area.x, title_row, "Overview", t.title().bg(bg));
                let meta = format!(
                    "{} · {}",
                    plural(s.counts.accounts, "account", "accounts"),
                    plural(s.counts.providers, "provider", "providers")
                );
                buf.set_string(
                    area.right().saturating_sub(width(&meta) as u16),
                    title_row,
                    &meta,
                    t.faint().bg(bg),
                );
                let lines = Self::overview_lines(w);
                self.detail_scroll.set_content(lines.len());
                self.detail_scroll.set_viewport(body.height as usize);
                for (k, i) in self.detail_scroll.visible_range().enumerate() {
                    let (text, tone) = &lines[i];
                    buf.set_string(
                        body.x,
                        body.y + k as u16,
                        truncate(text, body.width as usize),
                        Style::new().fg(t.tone(*tone)).bg(bg),
                    );
                }
            }
            Some(a) => {
                let title = a.title();
                buf.set_string(
                    area.x,
                    title_row,
                    truncate(&title, area.width.saturating_sub(20) as usize),
                    t.title().bg(bg),
                );
                let meta = a.status_word();
                buf.set_string(
                    area.right().saturating_sub(width(meta) as u16),
                    title_row,
                    meta,
                    t.faint().bg(bg),
                );
                let lines = Self::account_lines(&a, w);
                self.detail_scroll.set_content(lines.len());
                self.detail_scroll.set_viewport(body.height as usize);
                for (k, i) in self.detail_scroll.visible_range().enumerate() {
                    let (text, tone, meter) = &lines[i];
                    let y = body.y + k as u16;
                    match meter {
                        Some((pct, mt)) => {
                            let (label, rest) = text.split_once('\u{1}').unwrap_or((text, ""));
                            buf.set_string(body.x, y, fit(label, 15), t.primary().bg(bg));
                            let mx = body.x + 16;
                            Meter::new(Some(*pct))
                                .value(format!("{pct:>3}%"))
                                .tone(*mt)
                                .visual(MeterVisual::Block)
                                .render(Rect::new(mx, y, meter_w + 6, 1), buf, ctx, bg);
                            let vx = mx + meter_w + 8;
                            if vx < body.right() {
                                buf.set_string(
                                    vx,
                                    y,
                                    truncate(rest, body.right().saturating_sub(vx) as usize),
                                    t.muted().bg(bg),
                                );
                            }
                        }
                        None => {
                            let style = if text == "Limits" {
                                t.secondary().add_modifier(Modifier::BOLD)
                            } else {
                                Style::new().fg(t.tone(*tone))
                            };
                            buf.set_string(
                                body.x,
                                y,
                                truncate(text, body.width as usize),
                                style.bg(bg),
                            );
                        }
                    }
                }
            }
        }
        if self.detail_scroll.overflows() {
            scrollbar::render_vertical(
                Rect::new(area.right() - 1, body.y, 1, body.height),
                buf,
                ctx,
                DETAIL,
                &self.detail_scroll,
                focused,
            );
        }
    }
}

impl Screen for UsageScreen {
    fn enter(&mut self, w: &mut World, cx: &mut Cx) {
        self.build_rows(w);
        self.show_detail = false;
        cx.focus.focus(LIST);
    }

    fn primary_focus(&self) -> Option<WidgetId> {
        Some(LIST)
    }

    fn animating(&self, w: &World) -> bool {
        w.accounts
            .accounts
            .iter()
            .any(|a| a.usage.freshness.phase == Freshness::Refreshing)
    }

    fn on_tick(&mut self, w: &mut World, _cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        if self.refreshing && !self.animating(w) {
            self.refreshing = false;
            w.last_refresh_secs = w.now_secs();
            return Outcome::Changed;
        }
        if self.animating(w) {
            Outcome::Changed
        } else {
            Outcome::Ignored
        }
    }

    fn on_key(&mut self, key: &Key, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        let in_detail = cx.focus.is(DETAIL);
        match key.code {
            KeyCode::Up | KeyCode::Char('k') if !in_detail => {
                self.move_cursor(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') if !in_detail => {
                self.move_cursor(1);
                Outcome::Changed
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.detail_scroll.scroll_by(-1);
                Outcome::Changed
            }
            KeyCode::Down | KeyCode::Char('j') => {
                self.detail_scroll.scroll_by(1);
                Outcome::Changed
            }
            KeyCode::PageUp => {
                self.detail_scroll.page_up();
                Outcome::Changed
            }
            KeyCode::PageDown => {
                self.detail_scroll.page_down();
                Outcome::Changed
            }
            KeyCode::Enter => {
                self.show_detail = true;
                cx.focus.focus(DETAIL);
                Outcome::Changed
            }
            KeyCode::Char('r') if key.plain() => {
                self.refresh(w, cx);
                Outcome::Changed
            }
            KeyCode::Char('m') if key.plain() => {
                cx.go(Go::Accounts {
                    select: self.selected.clone(),
                });
                Outcome::Changed
            }
            KeyCode::Esc | KeyCode::Char('q') => {
                if self.show_detail || in_detail {
                    self.show_detail = false;
                    cx.focus.focus(LIST);
                    return Outcome::Changed;
                }
                cx.go(Go::Manager);
                Outcome::Changed
            }
            _ => Outcome::Ignored,
        }
    }

    fn on_click(&mut self, id: WidgetId, pos: Position, w: &mut World, cx: &mut Cx) -> Outcome {
        self.build_rows(w);
        for i in self.scroll.visible_range() {
            if LIST.child(i) == id {
                match &self.rows[i] {
                    Row::Account(a) => self.selected = Some(a.clone()),
                    Row::Overview => self.selected = None,
                    Row::Heading(_) => {}
                }
                self.detail_scroll.jump_start();
                cx.focus.focus(LIST);
                return Outcome::Changed;
            }
        }
        if id == DETAIL {
            cx.focus.focus(DETAIL);
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(LIST) {
            let track = Rect::new(
                self.list_area.right() - 1,
                self.list_area.y,
                1,
                self.list_area.height,
            );
            self.scroll
                .scroll_to(scrollbar::offset_for_click(track, pos, &self.scroll));
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(DETAIL) {
            let track = Rect::new(
                self.detail_area.right() - 1,
                self.detail_area.y + 2,
                1,
                self.detail_area.height.saturating_sub(2),
            );
            self.detail_scroll.scroll_to(scrollbar::offset_for_click(
                track,
                pos,
                &self.detail_scroll,
            ));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_wheel(&mut self, id: WidgetId, delta: i32, _pos: Position, _w: &mut World) -> Outcome {
        if id == LIST || id == scrollbar::id_for(LIST) {
            self.scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        if id == DETAIL || id == scrollbar::id_for(DETAIL) {
            self.detail_scroll.scroll_by(delta as isize);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn on_msg(&mut self, msg: &Msg, w: &mut World, _cx: &mut Cx) -> Outcome {
        if let Msg::AccountRefreshed { account } = msg {
            // the projection reloads in place; the Center owns the outcome table
            let now = w.now_secs();
            if let Some(a) = w.accounts.get_mut(account)
                && a.usage.freshness.phase == Freshness::Refreshing
            {
                a.usage.freshness = match a.issue.as_ref().map(|i| i.code) {
                    Some(IssueCode::RateLimited)
                    | Some(IssueCode::ProviderUnavailable)
                    | Some(IssueCode::CredentialFileMissing) => {
                        crate::domain::usage::FreshnessInfo::failed(
                            a.usage.freshness.last_good_secs,
                            a.usage.freshness.retry_secs,
                        )
                    }
                    _ => crate::domain::usage::FreshnessInfo::current(now),
                };
                a.last_refresh_secs = Some(now);
            }
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx, w: &World) {
        self.build_rows(w);
        let t = ctx.theme;
        let n = w
            .accounts
            .accounts
            .iter()
            .filter(|a| a.usage.freshness.phase == Freshness::Refreshing)
            .count();
        let meta = if n > 0 {
            format!("{} refreshing {n}", spinner_frame(w.now_ms() as u64 / 80))
        } else {
            format!("broker · {}", w.clock.ago(w.last_refresh_secs))
        };
        let focused_any = ctx.interaction.focused(LIST) || ctx.interaction.focused(DETAIL);
        let inner = Panel::framed(Some("Usage · read-only"))
            .focused(focused_any)
            .meta(&meta)
            .render(area, buf, t);
        if w.accounts.accounts.is_empty() {
            let e = EmptyState::new("No providers configured.")
                .hint("Press R to refresh. · c registers an account");
            empty::render(inner, buf, t, &e, t.canvas);
            ctx.control(LIST, Rect::new(inner.x, inner.y, 1, 1), false);
            return;
        }
        if area.width < 100 {
            if self.show_detail {
                self.draw_detail(inner, buf, ctx, w);
                ctx.control(LIST, Rect::ZERO, false);
            } else {
                self.draw_list(inner, buf, ctx, w);
                ctx.control(DETAIL, Rect::ZERO, false);
            }
            return;
        }
        let list_w = (inner.width * 34 / 100).clamp(28, 40);
        let list = Rect::new(inner.x, inner.y, list_w, inner.height);
        let detail = Rect::new(
            inner.x + list_w + 2,
            inner.y,
            inner.width.saturating_sub(list_w + 2),
            inner.height,
        );
        self.draw_list(list, buf, ctx, w);
        self.draw_detail(detail, buf, ctx, w);
    }

    fn hints(&self, focus: Option<WidgetId>, _w: &World) -> Vec<Hint> {
        if focus == Some(DETAIL) {
            return vec![
                hint("↑↓", "Scroll"),
                hint("r", "Refresh"),
                hint("m", "Manage in Accounts"),
                hint("Esc", "Back to list"),
            ];
        }
        vec![
            hint("↑↓", "Move"),
            hint("Enter", "Detail"),
            hint("r", "Refresh"),
            hint("m", "Manage in Accounts"),
            hint("Esc", "Close"),
        ]
    }

    fn crumb(&self, w: &World) -> String {
        match self.selected.as_ref().and_then(|id| w.accounts.get(id)) {
            Some(a) => format!("Usage › {} › {}", a.surface.surface_name(), a.display_name),
            None => "Usage › Overview".into(),
        }
    }
}
