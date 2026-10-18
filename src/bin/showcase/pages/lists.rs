use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

use crate::app::PageId;
use crate::options::page_options;
use crate::pages::row_menu::{MENU_HINTS, OPEN_HINT, RowMenu, plain};
use crate::pages::template::{Section, Template};
use crate::pages::{Cx, Hint, Page, PageEvent};
use junie_tui::core::event::Outcome;
use junie_tui::core::id::{Part, WidgetId};
use junie_tui::core::intent::Phase;
use junie_tui::core::keymap::{KeyBinding, NavKeys, label_for};
use junie_tui::core::pointer::MouseTarget;
use junie_tui::core::scroll::TrackClick;
use junie_tui::core::selection::{CheckClick, ClickPolicy, SelectGesture};
use junie_tui::core::typeahead::TypeAhead;
use junie_tui::theme::Tone;
use junie_tui::ui::content::ContentState;
use junie_tui::ui::context::ContextRequest;
use junie_tui::ui::ctx::RenderCtx;
use junie_tui::ui::reveal::{Reveal, RevealMode};
use junie_tui::ui::specimen::{SpecimenState, StateGrid};
use junie_tui::ui::text;
use junie_tui::widgets::empty::EmptyState;
use junie_tui::widgets::list::{ListBox, ListCmd, ListItem, SelectMode};
use junie_tui::widgets::menu::MenuItem;
use junie_tui::widgets::panel::Panel;
use junie_tui::widgets::scrollbar;
use ratatui::crossterm::event::KeyCode;

const ID: WidgetId = WidgetId::of("lists");
const MENU: WidgetId = WidgetId::of("lists.menu");

pub const SINGLE: WidgetId = ID.sub("single");
pub const MULTI: WidgetId = ID.sub("multi");
pub const EMPTY: WidgetId = ID.sub("empty");

const FILES: [(&str, &str, bool); 12] = [
    ("src/api/auth.rs", "modified", false),
    ("src/api/billing.rs", "modified", false),
    ("src/db/schema.rs", "generated", true),
    ("tests/checkout.rs", "new", false),
    ("Cargo.lock", "locked", true),
    ("docs/webhooks.md", "modified", false),
    ("src/workers/mailer.rs", "modified", false),
    ("src/config.rs", "modified", false),
    ("README.md", "modified", false),
    ("src/main.rs", "modified", false),
    ("tests/auth_flow.rs", "new", false),
    ("src/db/pool.rs", "modified", false),
];

fn languages() -> Vec<ListItem> {
    crate::data::languages()
        .iter()
        .map(|l| ListItem::new(l))
        .collect()
}

fn files() -> Vec<ListItem> {
    FILES
        .iter()
        .map(|(n, m, d)| {
            let tone = match *m {
                "new" => Tone::Success,
                "modified" => Tone::Warning,
                _ => Tone::Muted,
            };
            ListItem::new(n).meta(m).meta_tone(tone).disabled(*d)
        })
        .collect()
}

pub struct ListsPage {
    template: Template,
    single: ListBox,
    multi: ListBox,
    empty: ListBox,

    menu: RowMenu<(WidgetId, usize)>,

    opts: Vec<usize>,
}

impl ListsPage {
    pub fn new() -> Self {
        let mut page = Self {
            template: Template::new(ID, states_grid()),
            single: ListBox::new(SINGLE, languages(), SelectMode::Single),
            multi: ListBox::new(MULTI, files(), SelectMode::Multi),
            empty: ListBox::new(EMPTY, vec![], SelectMode::Single),
            menu: RowMenu::default(),
            opts: page_options(PageId::Lists)
                .iter()
                .map(|o| o.default)
                .collect(),
        };
        page.single.chosen = Some(0);
        page.multi.set_checked(0, true);
        page.multi.set_checked(1, true);
        page.rebuild();
        page
    }

    #[allow(dead_code)]
    pub fn single(&self) -> &ListBox {
        &self.single
    }

    fn opt(&self, key: &str) -> &'static str {
        let table = page_options(PageId::Lists);
        let i = table
            .iter()
            .position(|o| o.key == key)
            .unwrap_or_else(|| panic!("no option {key}"));
        table[i].values[self.opts[i]]
    }

    fn build(&self, l: ListBox) -> ListBox {
        let mut l = l
            .nav(match self.opt("lists.nav") {
                "vim" => NavKeys::VIM,
                _ => NavKeys::default(),
            })
            .type_ahead(self.opt("lists.type_ahead") == "on")
            .click(
                match self.opt("lists.click") {
                    "single" => ClickPolicy::NAVIGATION,
                    _ => ClickPolicy::DATA,
                }
                .check(match self.opt("lists.check") {
                    "row" => CheckClick::Row,
                    _ => CheckClick::BoxOnly,
                }),
            )
            .toggle_mods(match self.opt("lists.toggle_mods") {
                "alt" => ratatui::crossterm::event::KeyModifiers::ALT,
                "any" => SelectGesture::TOGGLE_WITH_ALT,
                _ => SelectGesture::toggle_mods(),
            })
            .wrap(self.opt("lists.wrap") == "on")
            .skip_disabled(self.opt("lists.skip_disabled") == "on")
            .scrollbar_gap(self.opt("lists.scrollbar_gap").parse().unwrap_or(1))
            .reveal(match self.opt("lists.reveal") {
                "inline" => Reveal::default().mode(RevealMode::InlineWrap),
                "off" => Reveal::default().hover(false).keyboard(None),
                _ => Reveal::default(),
            })
            .track_click(match self.opt("lists.track_click") {
                "jump" => TrackClick::Jump,
                _ => TrackClick::Page,
            });
        if self.opt("lists.keymap") == "custom" {
            l = l.keymap(vec![
                (KeyBinding::char('t'), ListCmd::ToggleAll),
                (KeyBinding::ctrl('a'), ListCmd::SelectAll),
            ]);
        }
        l
    }

    fn rebuild(&mut self) {
        let keep = |old: &ListBox, mut new: ListBox| {
            new.cursor = old.cursor;
            new.chosen = old.chosen;
            new.selection = old.selection.clone();
            new
        };
        let single = self
            .build(ListBox::new(SINGLE, languages(), SelectMode::Single))
            .content(match self.opt("lists.content") {
                "loading" => ContentState::loading("Loading languages…"),
                "error" => ContentState::error(
                    EmptyState::new("Could not load languages")
                        .hint("The registry did not answer; try again later"),
                ),
                _ => ContentState::Ready,
            });
        self.single = keep(&self.single, single);
        let multi = self.build(ListBox::new(MULTI, files(), SelectMode::Multi));
        self.multi = keep(&self.multi, multi);
        let empty = self.build(ListBox::new(EMPTY, vec![], SelectMode::Single));
        self.empty = if self.opt("lists.empty") == "state" {
            empty.empty(
                EmptyState::new("No results for “retry”")
                    .hint("Try a shorter word, or search every branch"),
            )
        } else {
            empty.empty_text("No results for “retry”")
        };
    }

    fn lists(&mut self) -> [&mut ListBox; 3] {
        [&mut self.single, &mut self.multi, &mut self.empty]
    }

    fn list_mut(&mut self, id: WidgetId) -> Option<&mut ListBox> {
        self.lists().into_iter().find(|l| l.id == id)
    }

    fn readout(&self) -> Vec<String> {
        let l = &self.multi;
        let nav = if l.nav == NavKeys::VIM {
            "vim"
        } else {
            "default"
        };
        let key = |c| label_for(&l.keymap, c).unwrap_or_else(|| "none".into());
        use ratatui::crossterm::event::KeyModifiers as M;

        let toggle = if l.toggle_mods == SelectGesture::toggle_mods() {
            "platform".to_owned()
        } else {
            [(M::CONTROL, "Ctrl"), (M::ALT, "Alt"), (M::SUPER, "Cmd")]
                .iter()
                .filter(|(m, _)| l.toggle_mods.contains(*m))
                .map(|(_, n)| *n)
                .collect::<Vec<_>>()
                .join("/")
        };
        vec![
            format!(
                "nav {nav} · type-ahead {} (TypeAhead, {} ms pause) · click {:?}, check {:?} · wrap {} · skip_disabled {}",
                if l.type_ahead { "on" } else { "off" },
                TypeAhead::DEFAULT_TIMEOUT.as_millis(),
                l.click.activate,
                l.click.check,
                l.wrap,
                l.skip_disabled,
            ),
            format!(
                "toggle {toggle}+click · scrollbar_gap {} · track_click {:?} · reveal {} · {} ListCmd::ToggleAll · {} ListCmd::SelectAll",
                l.scrollbar_gap,
                l.scroll.track_click.unwrap_or_default(),
                self.opt("lists.reveal"),
                key(ListCmd::ToggleAll),
                key(ListCmd::SelectAll),
            ),
        ]
    }

    fn row_items(l: &ListBox, row: usize) -> Vec<MenuItem> {
        let item = &l.items[row];
        match l.mode {
            SelectMode::Single => {
                let choose = MenuItem::new("Choose").shortcut("Enter");
                vec![if item.disabled {
                    choose.reason("Unavailable")
                } else if l.chosen == Some(row) {
                    choose.reason("Already chosen")
                } else {
                    choose
                }]
            }
            SelectMode::Multi => {
                let toggle = MenuItem::new(if l.is_checked(row) {
                    "Exclude"
                } else {
                    "Include"
                })
                .shortcut("Space");
                let toggle = if item.disabled {
                    toggle.reason(format!("{} file", item.meta.as_deref().unwrap_or("Locked")))
                } else {
                    toggle
                };
                let all = (0..l.items.len())
                    .filter(|&i| !l.items[i].disabled)
                    .all(|i| l.is_checked(i));
                let key = junie_tui::core::keymap::hint_key(&l.keymap, ListCmd::ToggleAll);
                let all = MenuItem::new(if all { "Select none" } else { "Select all" });
                vec![
                    toggle.separator(),
                    match key {
                        Some(k) => all.shortcut(k),
                        None => all,
                    },
                ]
            }
        }
    }

    fn open_menu(&mut self, id: WidgetId, req: ContextRequest<usize>) {
        let Some(l) = self.list_mut(id) else {
            return;
        };
        let title = l.items[req.target].label.clone();
        let items = Self::row_items(l, req.target);
        self.menu
            .open(req.menu(MENU, items).title(title), (id, req.target));
    }

    fn choose(&mut self, (id, row): (WidgetId, usize), i: usize) -> Outcome {
        let Some(l) = self.list_mut(id) else {
            return Outcome::Consumed;
        };
        l.cursor = row;
        match i {
            0 => l.activate(row),
            _ => {
                let key =
                    junie_tui::core::keymap::deliverable_binding_for(&l.keymap, ListCmd::ToggleAll)
                        .map_or(plain(KeyCode::Char('a')), |b| junie_tui::core::event::Key {
                            code: b.code,
                            mods: b.mods,
                        });
                l.on_key(&key)
            }
        }
    }

    fn render_playground(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let t = ctx.theme;
        let third = area.width / 3;
        let h = area.height.min(18);
        let cols = [
            Rect::new(area.x, area.y, third.saturating_sub(1), h),
            Rect::new(area.x + third + 1, area.y, third.saturating_sub(1), h),
            Rect::new(
                area.x + 2 * third + 2,
                area.y,
                area.width.saturating_sub(2 * third + 2),
                h,
            ),
        ];
        let chosen = self
            .single
            .chosen
            .map(|i| self.single.items[i].label.clone())
            .unwrap_or_default();
        let panel = Panel::card(Some("Language")).focused(ctx.interaction.focused(self.single.id));
        let bg = panel.bg(t);
        let inner = panel.render(cols[0], buf, t);
        buf.set_string(
            inner.x,
            inner.y,
            format!("Chosen: {chosen}"),
            t.muted().bg(bg),
        );
        self.single.render(
            Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(2),
            ),
            buf,
            ctx,
            bg,
        );
        panel.draw_meta(
            cols[0],
            buf,
            t,
            &scrollbar::position_label(&self.single.scroll),
        );

        let count = format!("{} selected", self.multi.checked_count());
        let panel = Panel::card(Some("Files to include"))
            .meta(&count)
            .focused(ctx.interaction.focused(self.multi.id));
        let bg = panel.bg(t);
        let inner = panel.render(cols[1], buf, t);
        let all = junie_tui::core::keymap::hint_key(&self.multi.keymap, ListCmd::ToggleAll)
            .unwrap_or("—");
        buf.set_string(
            inner.x,
            inner.y,
            text::truncate(
                &format!("Space toggle · {all} all · Shift+↓ range"),
                inner.width as usize,
            ),
            t.muted().bg(bg),
        );
        self.multi.render(
            Rect::new(
                inner.x,
                inner.y + 2,
                inner.width,
                inner.height.saturating_sub(2),
            ),
            buf,
            ctx,
            bg,
        );

        let panel = Panel::card(Some("Search results"));
        let bg = panel.bg(t);
        let inner = panel.render(cols[2], buf, t);
        self.empty.render(inner, buf, ctx, bg);

        let mut y = area.y + h + 1;
        for line in self.readout() {
            for l in text::wrap(&line, area.width as usize) {
                if y >= area.bottom() {
                    break;
                }
                buf.set_string(area.x, y, l, t.faint().bg(t.canvas));
                y += 1;
            }
        }
        self.menu.render(area, buf, ctx);
    }
}

fn states_grid() -> StateGrid {
    use SpecimenState::*;
    let id = ID.sub("states");
    let mut single = ListBox::new(id.child(0), languages(), SelectMode::Single);
    single.items.truncate(3);

    let mut multi = ListBox::new(id.child(1), files(), SelectMode::Multi);
    multi.items.truncate(3);

    StateGrid::new(id)
        .rows(&[
            Rest,
            Hover,
            Pressed,
            Focused,
            FocusedHover,
            Selected,
            Disabled,
            Empty,
            Loading,
            Error,
            Overflow,
            Narrow,
        ])
        .column_width(30)
        .column("Single", single)
        .column("Multi", multi)
}

impl Page for ListsPage {
    fn title(&self) -> &'static str {
        "Lists"
    }
    fn blurb(&self) -> &'static str {
        "Single and multiple selection, disabled items, scrolling, empty state"
    }

    fn render(&mut self, area: Rect, buf: &mut Buffer, ctx: &mut RenderCtx) {
        let body = self.template.render(area, buf, ctx);
        match self.template.section() {
            Section::Playground => self.render_playground(body, buf, ctx),
            Section::Options => self.template.render_options(body, buf, ctx),
            Section::States => self.template.render_states(body, buf, ctx),
        }
    }

    fn layer_open(&self) -> bool {
        self.menu.hides_page_focus() || self.template.popup_open()
    }

    fn handle(&mut self, ev: &PageEvent, cx: &mut Cx) -> Outcome {
        if let Some((out, chosen)) = self.menu.handle(ev) {
            return match chosen {
                Some((target, i)) => self.choose(target, i).or(out),
                None => out,
            };
        }
        if let Some(o) = self.template.handle(ev, cx) {
            return o;
        }
        match ev {
            PageEvent::Key(key) => {
                let Some(f) = cx.focus.current() else {
                    return Outcome::Ignored;
                };
                let cfg = self.menu.config;
                let Some(l) = self.list_mut(f) else {
                    return Outcome::Ignored;
                };
                if let Some(req) = cfg.on_key(l, key) {
                    self.open_menu(f, req);
                    return Outcome::Changed;
                }
                l.on_key(key)
            }
            PageEvent::Pointer(ev) if ev.is(Phase::Secondary) => {
                let cfg = self.menu.config;
                let hit = self
                    .lists()
                    .into_iter()
                    .find_map(|l| cfg.on_secondary(l, ev).map(|req| (l.id, req)));
                let Some((list, req)) = hit else {
                    return Outcome::Ignored;
                };
                cx.focus.focus(list);
                self.open_menu(list, req);
                Outcome::Changed
            }
            PageEvent::Pointer(ev) => {
                for l in self.lists() {
                    if let Some(r) = l.route(ev) {
                        if ev.is(Phase::Click) && ev.index_of(Part::ROW).is_some() {
                            cx.focus.focus(l.id);
                        }
                        return r.outcome();
                    }
                }
                Outcome::Ignored
            }
            _ => Outcome::Ignored,
        }
    }

    fn hints(&self, focus: Option<WidgetId>) -> Vec<Hint> {
        if self.menu.is_open() {
            return MENU_HINTS.to_vec();
        }
        if let Some(h) = self.template.hints(focus) {
            return h;
        }

        let l = if focus == Some(self.multi.id) {
            &self.multi
        } else {
            &self.single
        };
        let mut hints = Vec::new();
        if let Some(k) = l.nav.move_hint(l.type_ahead) {
            hints.push((k, "Move"));
        }
        if focus == Some(self.multi.id) {
            hints.push(("Space", "Toggle"));
            if let Some(k) = junie_tui::core::keymap::hint_key(&l.keymap, ListCmd::ToggleAll) {
                hints.push((k, "All / none"));
            }
            hints.push(("Shift+↓", "Range"));
        } else {
            hints.push(("Enter", "Choose"));
            if let Some(k) = l.nav.ends_hint(l.type_ahead) {
                hints.push((k, "Ends"));
            }
        }
        hints.push(OPEN_HINT);
        hints
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        let i = page_options(PageId::Lists)
            .iter()
            .position(|o| o.key == key)?;
        Some(self.opts[i])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let table = page_options(PageId::Lists);
        if let Some(i) = table.iter().position(|o| o.key == key) {
            self.opts[i] = value.min(table[i].values.len() - 1);
            self.rebuild();
        }
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }

    fn template(&self) -> Option<&Template> {
        Some(&self.template)
    }

    fn template_mut(&mut self) -> Option<&mut Template> {
        Some(&mut self.template)
    }
}
