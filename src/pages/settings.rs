//! Composed screen: project settings with tabs, a form, an editable
//! members table with a destructive dialog, and an environment list with a
//! prompt dialog.

use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Rect};

use crate::core::event::Outcome;
use crate::core::id::WidgetId;
use crate::pages::{Hint, Page, PageCtx, PageEvent};
use crate::ui::ctx::RenderCtx;
use crate::widgets::button::{Button, row_layout};
use crate::widgets::choice::{RadioGroup, Toggle};
use crate::widgets::dialog::{Dialog, DialogResult};
use crate::widgets::input::{InputEvent, TextInput};
use crate::widgets::list::{ListBox, ListItem, SelectMode};
use crate::widgets::panel::Panel;
use crate::widgets::scrollbar;
use crate::widgets::table::{Cell, Column, DataTable, TableEvent, Tone};
use crate::widgets::tabs::Tabs;
use crate::widgets::textarea::TextArea;

const ID: WidgetId = WidgetId::of("settings");
const REMOVE_DLG: WidgetId = ID.sub("remove-dialog");
const ADD_DLG: WidgetId = ID.sub("add-dialog");
const SAVE_DLG: WidgetId = ID.sub("save-dialog");

fn role_validator(col: usize, s: &str) -> Option<String> {
    match col {
        2 if !matches!(s, "Owner" | "Admin" | "Member" | "Viewer") => {
            Some("Role: Owner, Admin, Member or Viewer".into())
        }
        0 if s.trim().is_empty() => Some("Name required".into()),
        _ => None,
    }
}

fn var_validator(s: &str) -> Option<String> {
    if s.is_empty() {
        Some("Required".into())
    } else if !s
        .chars()
        .all(|c| c.is_ascii_uppercase() || c == '_' || c.is_ascii_digit())
    {
        Some("Use UPPER_SNAKE_CASE".into())
    } else {
        None
    }
}

pub struct SettingsPage {
    tabs: Tabs,
    // general
    name: TextInput,
    description: TextArea,
    visibility: RadioGroup,
    auto_merge: Toggle,
    protect: Toggle,
    save: Button,
    dirty: bool,
    // members
    members: DataTable,
    remove: Button,
    invite: Button,
    // env
    env: ListBox,
    add_var: Button,
    remove_vars: Button,
}

impl SettingsPage {
    pub fn new() -> Self {
        let members = [
            ("Mira Okafor", "mira@acme.dev", "Owner", "today"),
            ("Jonas Weber", "jonas@acme.dev", "Admin", "2 h ago"),
            ("Ana Costa", "ana@acme.dev", "Member", "yesterday"),
            ("Kai Tanaka", "kai@acme.dev", "Member", "3 d ago"),
            ("Sofia Rossi", "sofia@acme.dev", "Viewer", "never"),
            ("deploy-bot", "bot@acme.dev", "Member", "1 m ago"),
        ];
        let rows = members
            .iter()
            .map(|(n, e, r, l)| {
                vec![
                    Cell::new(*n),
                    Cell::new(*e).tone(Tone::Muted),
                    Cell::new(*r),
                    Cell::new(*l).tone(Tone::Muted),
                ]
            })
            .collect();
        let members = DataTable::new(
            ID.sub("members"),
            vec![
                Column::new("Name", Constraint::Min(14)).editable(),
                Column::new("Email", Constraint::Length(18)),
                Column::new("Role", Constraint::Length(8)).editable(),
                Column::new("Last active", Constraint::Length(11)).right(),
            ],
            rows,
        )
        .cell_nav(true)
        .validator(role_validator);
        let env_items = [
            ("DATABASE_URL", "postgres://…"),
            ("REDIS_URL", "redis://…"),
            ("STRIPE_KEY", "sk_live_…"),
            ("LOG_LEVEL", "info"),
            ("FEATURE_FLAGS", "beta,otel"),
        ]
        .iter()
        .map(|(k, v)| ListItem::new(k).meta(v))
        .collect();
        Self {
            tabs: Tabs::new(ID.sub("tabs"), &["General", "Members", "Environment"]),
            name: TextInput::new(ID.sub("name"), "Project name")
                .value("payments-gateway")
                .required(true),
            description: TextArea::new(ID.sub("desc"), "Description", 3)
                .value("Handles checkout, invoicing and refunds for the storefront."),
            visibility: RadioGroup::new(
                ID.sub("vis"),
                "Visibility",
                &["Private", "Internal", "Public"],
                0,
            ),
            auto_merge: Toggle::new(ID.sub("automerge"), "Auto-merge approved PRs", false),
            protect: Toggle::new(ID.sub("protect"), "Protect main branch", true),
            save: Button::primary(ID.sub("save"), "Save changes"),
            dirty: false,
            members,
            remove: Button::danger(ID.sub("remove"), "Remove…"),
            invite: Button::secondary(ID.sub("invite"), "Invite"),
            env: ListBox::new(ID.sub("env"), env_items, SelectMode::Multi)
                .empty_text("No variables defined"),
            add_var: Button::primary(ID.sub("addvar"), "Add variable…"),
            remove_vars: Button::danger(ID.sub("rmvars"), "Remove selected"),
        }
    }

    fn selected_member(&self) -> Option<usize> {
        if self.members.is_empty() {
            None
        } else {
            Some(self.members.source_row(self.members.cursor_row))
        }
    }
}

impl Page for SettingsPage {
    fn title(&self) -> &'static str {
        "Project settings"
    }
    fn blurb(&self) -> &'static str {
        "Composed: tabs, form, editable table, list, dialogs"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        self.tabs
            .render(Rect::new(area.x, area.y, area.width, 2), buf, ctx, t.canvas);
        let body = Rect::new(
            area.x,
            area.y + 3,
            area.width,
            area.height.saturating_sub(3),
        );
        match self.tabs.active {
            0 => {
                let title = if self.dirty {
                    "General · unsaved"
                } else {
                    "General"
                };
                let panel = Panel::card(Some(title));
                let bg = panel.bg(t);
                let inner = panel.render(
                    Rect::new(body.x, body.y, body.width, body.height.min(16)),
                    buf,
                    t,
                );
                let (l, r) = crate::pages::layout::columns(inner, inner.width / 2 - 2, 4);
                self.name.render(
                    Rect::new(l.x, l.y, l.width, TextInput::HEIGHT),
                    buf,
                    ctx,
                    bg,
                );
                self.description.render(
                    Rect::new(l.x, l.y + 3, l.width, self.description.height()),
                    buf,
                    ctx,
                    bg,
                );
                self.visibility.render(
                    Rect::new(r.x, r.y, r.width, self.visibility.height()),
                    buf,
                    ctx,
                    bg,
                );
                self.auto_merge
                    .render(Rect::new(r.x, r.y + 5, r.width, 1), buf, ctx, bg);
                self.protect
                    .render(Rect::new(r.x, r.y + 6, r.width, 1), buf, ctx, bg);
                let ay = inner.bottom().saturating_sub(1);
                self.save
                    .render(Rect::new(inner.x, ay, inner.width, 1), buf, ctx, bg);
                if !self.dirty {
                    buf.set_string(
                        inner.x + self.save.width() + 2,
                        ay,
                        "No changes",
                        t.faint().bg(bg),
                    );
                }
            }
            1 => {
                let pos = scrollbar::position_label(&self.members.scroll);
                let meta = match self.members.edit_error() {
                    Some(e) => e.to_owned(),
                    None if pos.is_empty() => format!("{} members", self.members.len()),
                    None => format!("{} members · {pos}", self.members.len()),
                };
                let panel = Panel::card(Some("Members"))
                    .meta(&meta)
                    .focused(ctx.interaction.focused(self.members.id));
                let bg = panel.bg(t);
                let th = (self.members.len() as u16 + 1).max(2);
                let card = Rect::new(body.x, body.y, body.width, (th + 5).min(body.height));
                let inner = panel.render(card, buf, t);
                let th = th.min(inner.height.saturating_sub(2));
                self.members
                    .render(Rect::new(inner.x, inner.y, inner.width, th), buf, ctx, bg);
                let ay = inner.bottom().saturating_sub(1);
                let rects = row_layout(
                    Rect::new(inner.x, ay, inner.width, 1),
                    &[self.invite.width(), self.remove.width()],
                    2,
                );
                self.invite.render(rects[0], buf, ctx, bg);
                self.remove.disabled = self.members.is_empty();
                self.remove.render(rects[1], buf, ctx, bg);
                if let Some(i) = self.selected_member() {
                    let s = format!("cursor: {}", self.members.rows[i][0].text);
                    buf.set_string(rects[1].right() + 2, ay, &s, t.faint().bg(bg));
                }
            }
            _ => {
                let count = self.env.checked_count();
                let meta = format!("{count} selected");
                let panel = Panel::card(Some("Environment variables")).meta(&meta);
                let bg = panel.bg(t);
                let lh = (self.env.items.len() as u16).max(1);
                let card = Rect::new(body.x, body.y, body.width, (lh + 5).min(body.height));
                let inner = panel.render(card, buf, t);
                let lh = lh.min(inner.height.saturating_sub(2));
                self.env.render(
                    Rect::new(inner.x, inner.y, inner.width.min(60), lh),
                    buf,
                    ctx,
                    bg,
                );
                let ay = inner.bottom().saturating_sub(1);
                let rects = row_layout(
                    Rect::new(inner.x, ay, inner.width, 1),
                    &[self.add_var.width(), self.remove_vars.width()],
                    2,
                );
                self.add_var.render(rects[0], buf, ctx, bg);
                self.remove_vars.disabled = count == 0;
                self.remove_vars.render(rects[1], buf, ctx, bg);
            }
        }
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut PageCtx) -> Outcome {
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                if f == self.tabs.id {
                    return self.tabs.on_key(key);
                }
                if key.ctrl_char('s') && self.tabs.active == 0 {
                    self.dirty = false;
                    cx.status("Settings saved ✓");
                    return Outcome::Changed;
                }
                let dirty = &mut self.dirty;
                macro_rules! input {
                    ($w:expr) => {{
                        let (o, iev) = $w.on_key(key);
                        match iev {
                            Some(InputEvent::CommittedTab { backward: false }) => cx.focus_next(),
                            Some(InputEvent::CommittedTab { backward: true }) => cx.focus_prev(),
                            Some(InputEvent::Changed) => *dirty = true,
                            _ => {}
                        }
                        return o;
                    }};
                }
                if f == self.name.id {
                    input!(self.name);
                }
                if f == self.description.id {
                    input!(self.description);
                }
                macro_rules! simple {
                    ($w:expr) => {{
                        let o = $w.on_key(key);
                        if o == Outcome::Changed {
                            *dirty = true;
                        }
                        return o;
                    }};
                }
                if f == self.visibility.id {
                    simple!(self.visibility);
                }
                if f == self.auto_merge.id {
                    simple!(self.auto_merge);
                }
                if f == self.protect.id {
                    simple!(self.protect);
                }
                if f == self.save.id {
                    let (o, act) = self.save.on_key(key);
                    if act {
                        if self.dirty {
                            cx.open(Dialog::confirm(SAVE_DLG, "Save settings?", "Changing visibility or branch protection applies to every open task.", "Save"));
                        } else {
                            cx.status("Nothing to save");
                        }
                    }
                    return o;
                }
                if f == self.members.id {
                    let (o, tev) = self.members.on_key(key);
                    match tev {
                        Some(TableEvent::Committed { .. }) => cx.status("Member updated"),
                        Some(TableEvent::LeaveForward) => cx.focus_next(),
                        Some(TableEvent::LeaveBackward) => cx.focus_prev(),
                        _ => {}
                    }
                    return o;
                }
                if f == self.remove.id {
                    let (o, act) = self.remove.on_key(key);
                    if act {
                        self.open_remove(cx);
                    }
                    return o;
                }
                if f == self.invite.id {
                    let (o, act) = self.invite.on_key(key);
                    if act {
                        cx.status("Invitations are sent from the web console");
                    }
                    return o;
                }
                if f == self.env.id {
                    return self.env.on_key(key);
                }
                if f == self.add_var.id {
                    let (o, act) = self.add_var.on_key(key);
                    if act {
                        self.open_add(cx);
                    }
                    return o;
                }
                if f == self.remove_vars.id {
                    let (o, act) = self.remove_vars.on_key(key);
                    if act {
                        self.remove_selected_vars(cx);
                    }
                    return o;
                }
                Outcome::Ignored
            }
            PageEvent::Paste(text) => {
                if self.name.editing {
                    return self.name.on_paste(text);
                }
                if self.description.editing {
                    return self.description.on_paste(text);
                }
                self.members.on_paste(text)
            }
            PageEvent::Click { id, pos } => {
                let id = *id;
                if let Some(i) = self.tabs.locate(id) {
                    cx.focus.focus(self.tabs.id);
                    return self.tabs.on_click(i);
                }
                if id == self.name.id {
                    let was = cx.focus.is(id);
                    cx.focus.focus(id);
                    return self.name.on_click(*pos, was);
                }
                if id == self.description.id {
                    let was = cx.focus.is(id);
                    cx.focus.focus(id);
                    return self.description.on_click(*pos, was);
                }
                for i in 0..self.visibility.options.len() {
                    if self.visibility.option_id(i) == id {
                        cx.focus.focus(self.visibility.id);
                        self.dirty = true;
                        return self.visibility.on_click(i);
                    }
                }
                if id == self.auto_merge.id {
                    self.dirty = true;
                    return self.auto_merge.on_click();
                }
                if id == self.protect.id {
                    self.dirty = true;
                    return self.protect.on_click();
                }
                if id == self.save.id {
                    if self.save.on_click() {
                        if self.dirty {
                            cx.open(Dialog::confirm(SAVE_DLG, "Save settings?", "Changing visibility or branch protection applies to every open task.", "Save"));
                        } else {
                            cx.status("Nothing to save");
                        }
                    }
                    return Outcome::Changed;
                }
                if let Some(c) = self.members.locate_header(id) {
                    cx.focus.focus(self.members.id);
                    return self.members.on_click_header(c);
                }
                if let Some((row, col)) = self.members.locate(id) {
                    cx.focus.focus(self.members.id);
                    return self.members.on_click_cell(row, col.unwrap_or(0), *pos).0;
                }
                if id == scrollbar::id_for(self.members.id) {
                    return self.members.on_scrollbar(*pos);
                }
                if id == self.remove.id {
                    if self.remove.on_click() {
                        self.open_remove(cx);
                    }
                    return Outcome::Changed;
                }
                if id == self.invite.id {
                    if self.invite.on_click() {
                        cx.status("Invitations are sent from the web console");
                    }
                    return Outcome::Changed;
                }
                if let Some(row) = self.env.locate(id) {
                    cx.focus.focus(self.env.id);
                    return self.env.on_click(row);
                }
                if id == scrollbar::id_for(self.env.id) {
                    return self.env.on_scrollbar(*pos);
                }
                if id == self.add_var.id {
                    if self.add_var.on_click() {
                        self.open_add(cx);
                    }
                    return Outcome::Changed;
                }
                if id == self.remove_vars.id {
                    if self.remove_vars.on_click() {
                        self.remove_selected_vars(cx);
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            PageEvent::Drag { pressed, pos } => {
                if *pressed == scrollbar::id_for(self.members.id) {
                    return self.members.on_scrollbar(*pos);
                }
                if *pressed == scrollbar::id_for(self.env.id) {
                    return self.env.on_scrollbar(*pos);
                }
                Outcome::Ignored
            }
            PageEvent::Wheel { id, delta } => {
                if self.members.owns(*id) {
                    return self.members.on_wheel(*delta);
                }
                if self.env.owns(*id) {
                    return self.env.on_wheel(*delta);
                }
                if self.description.id == *id {
                    return self.description.on_wheel(*delta);
                }
                Outcome::Ignored
            }
            PageEvent::DialogClosed { id, result, value } => {
                if *id == REMOVE_DLG {
                    if *result == DialogResult::Action(1) {
                        if let Some(i) = self.selected_member() {
                            let name = self.members.rows[i][0].text.clone();
                            let mut rows = self.members.rows.clone();
                            rows.remove(i);
                            self.members.set_rows(rows);
                            cx.status(format!("Removed {name}"));
                        }
                    } else {
                        cx.status("Kept member");
                    }
                    return Outcome::Changed;
                }
                if *id == ADD_DLG {
                    if *result == DialogResult::Action(1)
                        && let Some(name) = value.clone()
                    {
                        self.env.items.push(ListItem::new(&name).meta("(empty)"));
                        self.env.checked.push(false);
                        cx.status(format!("Added {name}"));
                    }
                    return Outcome::Changed;
                }
                if *id == SAVE_DLG {
                    if *result == DialogResult::Action(1) {
                        self.dirty = false;
                        cx.status("Settings saved ✓");
                    }
                    return Outcome::Changed;
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn editing(&self) -> bool {
        self.name.editing || self.description.editing || self.members.is_editing()
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.members.is_editing() {
            return vec![("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next cell")];
        }
        if self.name.editing || self.description.editing {
            return vec![("Enter", "Commit"), ("Esc", "Cancel"), ("Tab", "Next")];
        }
        match focus {
            Some(f) if f == self.tabs.id => vec![("← →", "Switch tab"), ("1 2 3", "Jump")],
            Some(f) if f == self.members.id => {
                vec![("↑ ↓ ← →", "Cell"), ("Enter", "Edit"), ("s", "Sort")]
            }
            Some(f) if f == self.env.id => vec![("Space", "Toggle"), ("a", "All")],
            _ => vec![("Enter", "Edit / activate"), ("Ctrl+S", "Save")],
        }
    }
}

impl SettingsPage {
    fn open_remove(&mut self, cx: &mut PageCtx) {
        let Some(i) = self.selected_member() else {
            return;
        };
        let name = self.members.rows[i][0].text.clone();
        let role = self.members.rows[i][2].text.clone();
        cx.open(Dialog::destructive(
            REMOVE_DLG,
            "Remove member?",
            &format!("{name} ({role}) will lose access to every task in this project immediately."),
            "Remove",
        ));
    }

    fn open_add(&mut self, cx: &mut PageCtx) {
        let input = TextInput::new(ADD_DLG.sub("name"), "Variable name")
            .placeholder("API_BASE_URL")
            .required(true)
            .validator(var_validator);
        cx.open(Dialog::prompt(ADD_DLG, "Add variable", input, "Add"));
    }

    fn remove_selected_vars(&mut self, cx: &mut PageCtx) {
        let before = self.env.items.len();
        let keep: Vec<bool> = self.env.checked.iter().map(|c| !*c).collect();
        let mut items = Vec::new();
        for (it, k) in self.env.items.drain(..).zip(keep) {
            if k {
                items.push(it);
            }
        }
        let n = items.len();
        self.env.items = items;
        self.env.checked = vec![false; n];
        self.env.cursor = self.env.cursor.min(n.saturating_sub(1));
        cx.status(format!("Removed {} variables", before - n));
    }
}
