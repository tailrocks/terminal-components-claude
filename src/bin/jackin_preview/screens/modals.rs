//! Host modal families composed from the design-system primitives: the
//! file browser, choice dialog (radio + buttons), form dialog (selects,
//! inputs, masked secrets, choose-buttons), the 1Password reference chain,
//! the read-only info dialog with copyable rows, and the help overlay.

use junie_tui::core::event::{Key, Outcome};
use junie_tui::core::focus::{Focus, FocusRing};
use junie_tui::core::id::WidgetId;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::text::truncate;
use junie_tui::widgets::button::{Button, row_layout_right};
use junie_tui::widgets::choice::Checkbox;
use junie_tui::widgets::input::{InputAction, TextInput};
use junie_tui::widgets::keyhint::{Hint, hint};
use junie_tui::widgets::list::{ListBox, ListItem, SelectMode};
use junie_tui::widgets::picker::{Picker, PickerAction, PickerItem, PickerStatus};
use junie_tui::widgets::progress::render_spinner;
use junie_tui::widgets::scrollbar;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::KeyCode;
use ratatui::layout::{Position, Rect};

use crate::domain::onepassword::OpReference;
use crate::sim::onepassword::{FieldKind, OpError, SimOnePassword};
use crate::sim::world::{FsEntry, World};

/// Dim the page (footer excluded), start the modal barrier and draw an
/// elevated rounded frame. Returns the inner content area.
#[allow(clippy::too_many_arguments)]
pub fn modal_frame(
    screen: Rect,
    buf: &mut Buffer,
    ctx: &mut RenderCtx,
    width_: u16,
    height: u16,
    title: &str,
    meta: Option<&str>,
    // the modal's dismissal policy: its close button shows when Esc may
    // close it (mouse #13)
    dismiss: junie_tui::ui::popup::Dismiss,
) -> (Rect, Rect) {
    // placed in the body, clear of the header and the footer, with the
    // dialog's margin around it
    junie_tui::ui::modal::Frame::new(width_, height)
        .title(title)
        .meta(meta)
        .placement(junie_tui::ui::popup::Placement::Center)
        .dismiss(dismiss)
        .render(screen, buf, ctx)
}

// ------------------------------------------------------------ file browser

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserResult {
    Chosen { path: String, readonly: bool },
    GitUrl(String),
    Cancelled,
}

pub struct FileBrowser {
    /// Which dismissals close it. Default `Dismiss::ESCAPE`.
    pub dismiss: junie_tui::ui::popup::Dismiss,
    pub title: String,
    pub cwd: String,
    pub path: TextInput,
    pub list: ListBox,
    pub entries: Vec<FsEntry>,
    pub readonly: Option<Checkbox>,
    pub git_url: Button,
    pub cancel: Button,
    pub choose: Button,
    /// Git URL mode: the field holds a URL and `Choose` resolves it.
    pub url_mode: bool,
    pub resolving: Option<(String, i64)>,
    pub error: Option<String>,
    pub dirs_only: bool,
    pub result: Option<BrowserResult>,
    pub area: Rect,
}

impl FileBrowser {
    pub fn new(
        id: WidgetId,
        title: &str,
        cwd: &str,
        with_readonly: bool,
        dirs_only: bool,
        w: &World,
    ) -> Self {
        let mut b = Self {
            dismiss: junie_tui::ui::popup::Dismiss::ESCAPE,
            title: title.to_owned(),
            cwd: cwd.to_owned(),
            path: TextInput::new(id.sub("path"), "Path").plain_label(),
            list: ListBox::new(id.sub("list"), vec![], SelectMode::Single)
                .empty_text("Empty folder"),
            entries: vec![],
            readonly: with_readonly.then(|| Checkbox::new(id.sub("ro"), "Mount read-only", false)),
            git_url: Button::secondary(id.sub("git"), "Git URL…"),
            cancel: Button::subtle(id.sub("cancel"), "Cancel"),
            choose: Button::primary(id.sub("choose"), "Choose"),
            url_mode: false,
            resolving: None,
            error: None,
            dirs_only,
            result: None,
            area: Rect::ZERO,
        };
        b.load(w);
        b
    }

    pub fn load(&mut self, w: &World) {
        let cwd = self.cwd.trim_end_matches('/').to_owned();
        let mut items: Vec<FsEntry> =
            w.fs.iter()
                .filter(|e| {
                    e.path
                        .rsplit_once('/')
                        .map(|(parent, _)| parent == cwd)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();
        items.sort_by(|a, b| b.dir.cmp(&a.dir).then(a.path.cmp(&b.path)));
        let mut rows = vec![ListItem::new("..").meta("parent")];
        for e in &items {
            let name = e.path.rsplit('/').next().unwrap_or(&e.path).to_owned();
            let label = if e.dir { format!("{name}/") } else { name };
            let meta = match (&e.git, e.meta.as_str()) {
                (Some(b), _) => format!("git · {b}"),
                (None, "") => String::new(),
                (None, m) => m.to_owned(),
            };
            let mut li = ListItem::new(&label);
            if !meta.is_empty() {
                li = li.meta(&meta);
            }
            li.disabled = self.dirs_only && !e.dir;
            rows.push(li);
        }
        self.entries = items;
        let cur = self.list.cursor.min(rows.len().saturating_sub(1));
        self.list = ListBox::new(self.list.id, rows, SelectMode::Single).empty_text("Empty folder");
        self.list.cursor = cur;
        self.path = TextInput::new(self.path.id, "Path")
            .plain_label()
            .value(&w.tilde(&self.cwd));
        self.error = None;
    }

    fn open_cursor(&mut self, w: &World) -> Outcome {
        let i = self.list.cursor;
        if i == 0 {
            if let Some((parent, _)) = self.cwd.rsplit_once('/')
                && !parent.is_empty()
            {
                self.cwd = parent.to_owned();
                self.list.cursor = 0;
                self.load(w);
            }
            return Outcome::Changed;
        }
        let Some(e) = self.entries.get(i - 1) else {
            return Outcome::Consumed;
        };
        if e.dir {
            self.cwd = e.path.clone();
            self.list.cursor = 0;
            self.load(w);
            Outcome::Changed
        } else {
            Outcome::Consumed
        }
    }

    fn choose(&mut self, w: &World) -> Outcome {
        if self.url_mode {
            let url = self.path.text().trim().to_owned();
            if url.is_empty() {
                self.error = Some("Enter a repository URL".into());
                return Outcome::Changed;
            }
            if !w.github.iter().any(|r| url.contains(&r.full_name)) {
                self.error = Some(format!("Could not resolve {url}: repository not found"));
                return Outcome::Changed;
            }
            self.result = Some(BrowserResult::GitUrl(url));
            return Outcome::Changed;
        }
        let i = self.list.cursor;
        let path = if i == 0 {
            self.cwd.clone()
        } else {
            match self.entries.get(i - 1) {
                Some(e) if e.dir => e.path.clone(),
                Some(_) if self.dirs_only => {
                    self.error = Some("Choose a folder, not a file".into());
                    return Outcome::Changed;
                }
                Some(e) => e.path.clone(),
                None => self.cwd.clone(),
            }
        };
        self.result = Some(BrowserResult::Chosen {
            path,
            readonly: self.readonly.as_ref().is_some_and(|c| c.checked),
        });
        Outcome::Changed
    }

    fn toggle_url_mode(&mut self, w: &World) {
        self.url_mode = !self.url_mode;
        if self.url_mode {
            self.path = TextInput::new(self.path.id, "Git URL")
                .plain_label()
                .placeholder("github.com/org/repo");
            self.git_url.label = "Browse folders".into();
        } else {
            self.git_url.label = "Git URL…".into();
            self.load(w);
        }
        self.error = None;
    }

    pub fn on_key(&mut self, key: &Key, focus: &mut Focus, ring: &FocusRing, w: &World) -> Outcome {
        let cur = focus.current();
        if cur == Some(self.path.id) {
            let editing = self.path.editing;
            let (o, ev) = self.path.on_key(key);
            match ev {
                Some(InputAction::Committed) => {
                    if self.url_mode {
                        return self.choose(w);
                    }
                    let typed = crate::sim::world::expand(&w.home, self.path.text().trim());
                    if w.fs.iter().any(|e| e.path == typed && e.dir) {
                        self.cwd = typed;
                        self.list.cursor = 0;
                        self.load(w);
                        focus.focus(self.list.id);
                    } else {
                        self.error = Some(format!("Folder not found: {}", self.path.text()));
                    }
                    return Outcome::Changed;
                }
                Some(InputAction::CommittedTab { backward }) => {
                    if backward {
                        focus.prev(ring)
                    } else {
                        focus.next(ring)
                    }
                    return Outcome::Changed;
                }
                _ => {}
            }
            if o.consumed() || editing {
                return o.or(Outcome::Consumed);
            }
        }
        if cur == Some(self.list.id) {
            match key.code {
                KeyCode::Enter | KeyCode::Right | KeyCode::Char('l') if key.plain() => {
                    return self.open_cursor(w);
                }
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Backspace if key.plain() => {
                    self.list.cursor = 0;
                    return self.open_cursor(w);
                }
                KeyCode::Char(' ') | KeyCode::Char('s') if key.plain() => return self.choose(w),
                KeyCode::Char('g') if key.plain() => {
                    self.toggle_url_mode(w);
                    focus.focus(self.path.id);
                    self.path.begin_edit();
                    return Outcome::Changed;
                }
                _ => {}
            }
            let o = self.list.on_key(key);
            if o.consumed() {
                return o;
            }
        }
        if let Some(c) = self.readonly.as_mut()
            && cur == Some(c.id)
        {
            let o = c.on_key(key);
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.git_url.id) {
            let (o, fired) = self.git_url.on_key(key);
            if fired {
                self.toggle_url_mode(w);
                focus.focus(self.path.id);
                self.path.begin_edit();
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.cancel.id) {
            let (o, fired) = self.cancel.on_key(key);
            if fired {
                self.result = Some(BrowserResult::Cancelled);
                return Outcome::Changed;
            }
            if o.consumed() {
                return o;
            }
        }
        if cur == Some(self.choose.id) {
            let (o, fired) = self.choose.on_key(key);
            if fired {
                return self.choose(w);
            }
            if o.consumed() {
                return o;
            }
        }
        match key.code {
            KeyCode::Esc => {
                self.result = Some(BrowserResult::Cancelled);
                Outcome::Changed
            }
            KeyCode::Tab => {
                focus.next(ring);
                Outcome::Changed
            }
            KeyCode::BackTab => {
                focus.prev(ring);
                Outcome::Changed
            }
            KeyCode::Enter if cur == Some(self.list.id) => self.open_cursor(w),
            _ => Outcome::Consumed,
        }
    }

    /// A press on `id` with the click count it starts: the path field
    /// remembers it for the drag.
    pub fn on_press(&mut self, id: WidgetId, pos: Position, count: u8) -> Outcome {
        if id == self.path.id {
            return self.path.on_press_count(pos, count);
        }
        Outcome::Ignored
    }

    /// A drag held since a press on `pressed`: the path field selects.
    pub fn on_drag(&mut self, pressed: WidgetId, pos: Position) -> Outcome {
        if pressed == self.path.id {
            return self.path.on_drag(pos);
        }
        Outcome::Ignored
    }

    pub fn on_click(
        &mut self,
        id: WidgetId,
        pos: Position,
        clicks: u8,
        focus: &mut Focus,
        w: &World,
    ) -> Outcome {
        if id == self.path.id {
            focus.focus(id);
            return self.path.on_click_count(pos, clicks.max(1));
        }
        if let Some(i) = self.list.locate(id) {
            focus.focus(self.list.id);
            let was = self.list.cursor == i;
            self.list.cursor = i;
            self.list.chosen = Some(i);
            if was {
                return self.open_cursor(w);
            }
            return Outcome::Changed;
        }
        if id == scrollbar::id_for(self.list.id) {
            return self.list.on_scrollbar(pos);
        }
        if let Some(c) = self.readonly.as_mut()
            && c.id == id
        {
            focus.focus(id);
            return c.on_click();
        }
        if id == self.git_url.id {
            focus.focus(id);
            if self.git_url.on_click() {
                self.toggle_url_mode(w);
                focus.focus(self.path.id);
                self.path.begin_edit();
            }
            return Outcome::Changed;
        }
        if id == self.cancel.id && self.cancel.on_click() {
            self.result = Some(BrowserResult::Cancelled);
            return Outcome::Changed;
        }
        if id == self.choose.id && self.choose.on_click() {
            return self.choose(w);
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.list.on_wheel(delta)
    }

    pub fn on_paste(&mut self, text: &str) -> Outcome {
        self.path.on_paste(text)
    }

    pub fn is_editing(&self) -> bool {
        self.path.editing
    }

    pub fn tick(&mut self, now_ms: i64) -> Outcome {
        if let Some((url, at)) = &self.resolving
            && now_ms >= *at
        {
            let u = url.clone();
            self.resolving = None;
            self.result = Some(BrowserResult::GitUrl(u));
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    pub fn render(
        &mut self,
        screen: Rect,
        buf: &mut Buffer,
        ctx: &mut RenderCtx,
        stepper: Option<&str>,
    ) {
        let (w, h) = if screen.width < 100 {
            (
                screen.width.saturating_sub(4).min(76),
                screen.height.saturating_sub(4).min(20),
            )
        } else {
            (84, 22)
        };
        let (area, inner) = modal_frame(screen, buf, ctx, w, h, &self.title, None, self.dismiss);
        self.area = area;
        let t = ctx.theme;
        let bg = t.surface_elevated;
        let mut y = inner.y;
        if let Some(s) = stepper {
            buf.set_string(
                inner.x,
                y,
                truncate(s, inner.width as usize),
                t.muted().bg(bg),
            );
            y += 2;
        }
        self.path
            .render(Rect::new(inner.x - 1, y, inner.width + 1, 2), buf, ctx, bg);
        y += 3;
        let extra: u16 = 2 + u16::from(self.readonly.is_some()) + 1;
        let list_h = inner.bottom().saturating_sub(y + extra + 1);
        if self.url_mode {
            let lines = [
                "Enter a GitHub repository URL. The repository is cloned into the",
                "Construct at launch; the branch is chosen in the mount row.",
            ];
            for (i, l) in lines.iter().enumerate() {
                buf.set_string(inner.x + 1, y + i as u16, l, t.muted().bg(bg));
            }
            if let Some((url, _)) = &self.resolving {
                render_spinner(
                    Rect::new(inner.x + 1, y + 3, inner.width, 1),
                    buf,
                    ctx,
                    &format!("Resolving {url}…"),
                    bg,
                );
            }
        } else {
            self.list.render(
                Rect::new(inner.x - 1, y, inner.width + 1, list_h),
                buf,
                ctx,
                bg,
            );
        }
        y += list_h + 1;
        if let Some(c) = self.readonly.as_mut() {
            c.render(Rect::new(inner.x - 1, y, inner.width + 1, 1), buf, ctx, bg);
            y += 1;
        }
        if let Some(e) = &self.error {
            buf.set_string(
                inner.x,
                y,
                truncate(&format!("! {e}"), inner.width as usize),
                t.error_fg().bg(bg),
            );
        }
        let ay = inner.bottom().saturating_sub(1);
        let widths = [
            self.git_url.width(),
            self.cancel.width(),
            self.choose.width(),
        ];
        let rects = row_layout_right(Rect::new(inner.x, ay, inner.width, 1), &widths, 1);
        self.git_url.render(rects[0], buf, ctx, bg);
        self.cancel.render(rects[1], buf, ctx, bg);
        self.choose.render(rects[2], buf, ctx, bg);
        ctx.hits.register(self.path.id, self.path.area);
        for b in [&self.git_url, &self.cancel, &self.choose] {
            ctx.hits.register(b.id, b.area);
        }
        if let Some(c) = &self.readonly {
            ctx.hits.register(c.id, c.area);
        }
    }

    pub fn initial_focus(&self) -> WidgetId {
        self.list.id
    }
}

// ------------------------------------------------------- library dialogs

// The choice, form and info dialogs moved into the library; the paths
// below keep this app's call sites unchanged.
pub use junie_tui::widgets::choice_dialog::ChoiceDialog;
pub use junie_tui::widgets::form_dialog::{
    FieldValue, FormAction, FormDialog, FormField, FormFieldKind as FieldKindW, FormValues,
};
pub use junie_tui::widgets::info_dialog::{InfoDialog, InfoResult};

// ------------------------------------------------------- 1Password chain

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpStep {
    Account,
    Vault,
    Item,
    Field,
}

/// Account → vault → item → field, each stage a searchable picker fed by
/// the simulated service; only reference metadata leaves the flow.
pub struct OpFlow {
    /// Which dismissals close it. Default `Dismiss::LIGHT`.
    pub dismiss: junie_tui::ui::popup::Dismiss,
    pub step: OpStep,
    pub account: Option<String>,
    pub vault: Option<(String, String)>,
    pub item: Option<(String, String)>,
    pub picker: Picker,
    pub result: Option<Option<OpReference>>,
    pub loading_until: Option<i64>,
    pub last_error: Option<OpError>,
    /// Restricts the field stage to concealed fields.
    pub concealed_only: bool,
}

impl OpFlow {
    pub fn new(id: WidgetId, op: &SimOnePassword, now_ms: i64) -> Self {
        let mut f = Self {
            dismiss: junie_tui::ui::popup::Dismiss::LIGHT,
            step: OpStep::Account,
            account: None,
            vault: None,
            item: None,
            picker: Picker::new(id.sub("picker"), "1Password"),
            result: None,
            loading_until: None,
            last_error: None,
            concealed_only: true,
        };
        f.picker.width = 64;
        // a single available account is chosen automatically
        let avail: Vec<&crate::sim::onepassword::OpAccount> = op
            .accounts
            .iter()
            .filter(|a| a.state == crate::sim::onepassword::OpAccountState::Available)
            .collect();
        if op.accounts.len() == 1 || (avail.len() == 1 && op.accounts.len() == 1) {
            f.account = Some(op.accounts[0].id.clone());
            f.step = OpStep::Vault;
        }
        f.begin_load(now_ms, op);
        f
    }

    fn crumb(&self) -> String {
        let mut parts = vec!["1Password".to_owned()];
        if let Some(a) = &self.account {
            parts.push(a.split('.').next().unwrap_or(a).to_owned());
        }
        if let Some((_, v)) = &self.vault {
            parts.push(v.clone());
        }
        if let Some((_, i)) = &self.item {
            parts.push(i.clone());
        }
        parts.join(" › ")
    }

    fn begin_load(&mut self, now_ms: i64, op: &SimOnePassword) {
        self.picker.query.clear();
        self.picker.title = match self.step {
            OpStep::Account => "Choose 1Password account".into(),
            OpStep::Vault => "Choose vault".into(),
            OpStep::Item => "Choose item".into(),
            OpStep::Field => "Choose field".into(),
        };
        self.picker.scope = Some(self.crumb());
        self.picker.placeholder = match self.step {
            OpStep::Account => "Search accounts…".into(),
            OpStep::Vault => "Search vaults…".into(),
            OpStep::Item => "Search items…".into(),
            OpStep::Field => "Search fields…".into(),
        };
        self.picker.searchable = true;
        self.picker.status = PickerStatus::Loading(match self.step {
            OpStep::Account => "loading accounts…".into(),
            OpStep::Vault => "loading vaults…".into(),
            OpStep::Item => format!(
                "loading items from {}…",
                self.vault.as_ref().map(|v| v.1.as_str()).unwrap_or("vault")
            ),
            OpStep::Field => format!(
                "loading {}…",
                self.item.as_ref().map(|i| i.1.as_str()).unwrap_or("item")
            ),
        });
        self.picker.set_items(vec![]);
        self.loading_until = Some(now_ms + op.latency_ms);
    }

    /// Called by the app on ticks: finish loading when due.
    pub fn tick(&mut self, now_ms: i64, op: &SimOnePassword) -> Outcome {
        if let Some(t) = self.loading_until
            && now_ms >= t
        {
            self.loading_until = None;
            self.refresh(op);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }

    fn set_error(&mut self, e: OpError) {
        let (msg, detail) = match &e {
            OpError::Locked => (
                "1Password is locked".to_owned(),
                Some("Unlock it in the 1Password app, then press r to retry".to_owned()),
            ),
            OpError::AuthorizationRequired { account } => (
                "Jackin is not authorized for 1Password CLI".to_owned(),
                Some(format!(
                    "Approve the request for {account} in 1Password, then press r to retry"
                )),
            ),
            OpError::PermissionDenied { vault } => (
                format!("No access to vault {vault}"),
                Some("Choose another vault".into()),
            ),
            other => (other.message(), None),
        };
        self.picker.status = PickerStatus::Error {
            message: msg,
            detail,
        };
        self.last_error = Some(e);
    }

    /// Rebuild rows for the current step and query.
    pub fn refresh(&mut self, op: &SimOnePassword) {
        self.picker.status = PickerStatus::Ready;
        self.last_error = None;
        let q = self.picker.query.to_lowercase();
        let matches = |s: &str| q.is_empty() || s.to_lowercase().contains(&q);
        let items: Vec<PickerItem> = match self.step {
            OpStep::Account => match op.list_accounts() {
                Ok(accts) => accts
                    .iter()
                    .filter(|a| matches(&a.id) || matches(&a.email))
                    .map(|a| PickerItem {
                        label: a.id.clone(),
                        detail: a.email.clone(),
                        glyph: "▪",
                        group: "",
                        tag: Some(match a.state {
                            crate::sim::onepassword::OpAccountState::Available => "signed in",
                            crate::sim::onepassword::OpAccountState::Locked => "locked",
                            crate::sim::onepassword::OpAccountState::AuthorizationRequired => {
                                "authorize"
                            }
                        }),
                        matched: vec![],
                        disabled: false,
                        key: String::new(),
                    })
                    .collect(),
                Err(e) => {
                    self.set_error(e);
                    return;
                }
            },
            OpStep::Vault => match op.list_vaults(self.account.as_deref().unwrap_or("")) {
                Ok(vaults) => vaults
                    .iter()
                    .filter(|v| matches(&v.name))
                    .map(|v| PickerItem {
                        label: v.name.clone(),
                        detail: format!("{} items", v.items.len()),
                        glyph: "▪",
                        group: "",
                        tag: Some(match v.access {
                            crate::sim::onepassword::VaultAccess::ReadWrite => "",
                            crate::sim::onepassword::VaultAccess::ReadOnly => "read-only",
                            crate::sim::onepassword::VaultAccess::Denied => "no access",
                        }),
                        matched: vec![],
                        disabled: false,
                        key: String::new(),
                    })
                    .collect(),
                Err(e) => {
                    self.set_error(e);
                    return;
                }
            },
            OpStep::Item => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                match op.list_items(self.account.as_deref().unwrap_or(""), &vid) {
                    Ok(items) => items
                        .iter()
                        .filter(|i| matches(&i.title))
                        .map(|i| PickerItem {
                            label: i.title.clone(),
                            detail: format!("{} · {} fields", i.category, i.fields.len()),
                            glyph: " ",
                            group: "",
                            tag: None,
                            matched: vec![],
                            disabled: false,
                            key: String::new(),
                        })
                        .collect(),
                    Err(e) => {
                        self.set_error(e);
                        return;
                    }
                }
            }
            OpStep::Field => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                let (iid, _) = self.item.clone().unwrap_or_default();
                match op.list_fields(self.account.as_deref().unwrap_or(""), &vid, &iid) {
                    Ok(fields) => fields
                        .iter()
                        .filter(|f| matches(&f.label))
                        .map(|f| PickerItem {
                            label: f.label.clone(),
                            detail: f.kind.label().to_owned(),
                            glyph: if f.kind == FieldKind::Concealed {
                                "•"
                            } else {
                                " "
                            },
                            group: "",
                            tag: None,
                            matched: vec![],
                            disabled: self.concealed_only && f.kind != FieldKind::Concealed,
                            key: String::new(),
                        })
                        .collect(),
                    Err(e) => {
                        self.set_error(e);
                        return;
                    }
                }
            }
        };
        if items.is_empty() {
            self.picker.empty_text = "No matches".into();
        }
        self.picker.set_items(items);
    }

    fn back(&mut self, now_ms: i64, op: &SimOnePassword) -> Outcome {
        match self.step {
            OpStep::Account => {
                self.result = Some(None);
            }
            OpStep::Vault => {
                if op.accounts.len() > 1 {
                    self.step = OpStep::Account;
                    self.account = None;
                    self.begin_load(now_ms, op);
                } else {
                    self.result = Some(None);
                }
            }
            OpStep::Item => {
                self.step = OpStep::Vault;
                self.vault = None;
                self.begin_load(now_ms, op);
            }
            OpStep::Field => {
                self.step = OpStep::Item;
                self.item = None;
                self.begin_load(now_ms, op);
            }
        }
        Outcome::Changed
    }

    fn choose(&mut self, i: usize, now_ms: i64, op: &SimOnePassword) -> Outcome {
        let Some(row) = self.picker.items.get(i).cloned() else {
            return Outcome::Consumed;
        };
        match self.step {
            OpStep::Account => {
                self.account = Some(row.label);
                self.step = OpStep::Vault;
            }
            OpStep::Vault => {
                let vid = op
                    .list_vaults(self.account.as_deref().unwrap_or(""))
                    .ok()
                    .and_then(|vs| {
                        vs.iter()
                            .find(|v| v.name == row.label)
                            .map(|v| v.id.clone())
                    })
                    .unwrap_or_default();
                self.vault = Some((vid, row.label));
                self.step = OpStep::Item;
            }
            OpStep::Item => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                let iid = op
                    .list_items(self.account.as_deref().unwrap_or(""), &vid)
                    .ok()
                    .and_then(|is| {
                        is.iter()
                            .find(|i| i.title == row.label)
                            .map(|i| i.id.clone())
                    })
                    .unwrap_or_default();
                self.item = Some((iid, row.label));
                self.step = OpStep::Field;
            }
            OpStep::Field => {
                let (vid, _) = self.vault.clone().unwrap_or_default();
                let (iid, _) = self.item.clone().unwrap_or_default();
                match op.reference(
                    self.account.as_deref().unwrap_or(""),
                    &vid,
                    &iid,
                    &row.label,
                ) {
                    Ok(r) => match op.describe(&r) {
                        Ok(_) => self.result = Some(Some(r)),
                        Err(e) => self.set_error(e),
                    },
                    Err(e) => self.set_error(e),
                }
                return Outcome::Changed;
            }
        }
        self.begin_load(now_ms, op);
        Outcome::Changed
    }

    pub fn on_key(&mut self, key: &Key, now_ms: i64, op: &SimOnePassword) -> Outcome {
        if key.is_char('r')
            && self.picker.status != PickerStatus::Ready
            && self.picker.query.is_empty()
        {
            self.begin_load(now_ms, op);
            return Outcome::Changed;
        }
        if self.loading_until.is_some() && !key.is(KeyCode::Esc) {
            return Outcome::Consumed;
        }
        let (o, ev) = self.picker.on_key(key);
        match ev {
            Some(PickerAction::QueryChanged) => {
                if (self.picker.status == PickerStatus::Ready
                    || matches!(self.picker.status, PickerStatus::Error { .. }))
                    && self.last_error.is_none()
                {
                    self.refresh(op);
                }
                Outcome::Changed
            }
            Some(PickerAction::Chosen(i)) | Some(PickerAction::ChosenAlt(i)) => {
                self.choose(i, now_ms, op)
            }
            Some(PickerAction::Back) | Some(PickerAction::Cancelled) => self.back(now_ms, op),
            _ => o.or(Outcome::Consumed),
        }
    }

    pub fn on_click(&mut self, id: WidgetId, now_ms: i64, op: &SimOnePassword) -> Outcome {
        if let Some(PickerAction::Chosen(i)) = self.picker.on_click(id) {
            return self.choose(i, now_ms, op);
        }
        Outcome::Consumed
    }

    pub fn on_wheel(&mut self, delta: i32) -> Outcome {
        self.picker.on_wheel(delta)
    }

    /// Footer hints for the current step (the shell's hint bar shows them).
    pub fn hints(&self) -> Vec<Hint> {
        match (self.step, &self.picker.status) {
            (_, PickerStatus::Error { .. }) => vec![hint("r", "Retry"), hint("Esc", "Back")],
            (OpStep::Field, _) => vec![
                hint("↑↓", "Move"),
                hint("Enter", "Save reference"),
                hint("Esc", "Back to items"),
            ],
            (OpStep::Account, _) => vec![
                hint("↑↓", "Move"),
                hint("Enter", "Choose"),
                hint("Esc", "Cancel"),
            ],
            _ => vec![
                hint("↑↓", "Move"),
                hint("Enter", "Choose"),
                hint("Backspace", "Back"),
                hint("Esc", "Back"),
            ],
        }
    }

    pub fn render(&mut self, screen: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        self.picker.draw(screen, buf, ctx);
    }
}

// ------------------------------------------------------------------ help

pub use junie_tui::widgets::help::HelpOverlay;
