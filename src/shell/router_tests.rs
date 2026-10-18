use std::time::Duration;

use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{KeyCode, KeyModifiers};
use ratatui::layout::Rect;

use super::*;
use crate::core::id::WidgetId;
use crate::theme::Theme;
use crate::ui::ctx::RenderCtx;
use crate::widgets::list::{ListBox, ListItem, SelectMode};
use crate::widgets::too_small::TooSmall;

use super::super::{App, Shell};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cmd {
    Delete,
    Run,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Took {
    Layer,
    Menu,
    Interrupt,
    Quit,
    Action(ShellAction<Cmd>),
    Chrome,
    Page,
    EscTop,
    Note(String),
    Pasted(String),
    Nothing,
}

struct Host {
    shell: Shell,
    keys: ShellKeys<Cmd>,
    layer: Option<bool>,
    menu: bool,
    capture: bool,
    interrupts: bool,
    dirty: bool,
    chrome: bool,

    page_takes: Vec<Key>,

    list: Option<ListBox>,
    esc_top: bool,
    took: Vec<Took>,
}

impl Host {
    fn new() -> Self {
        Self {
            shell: Shell::new(),
            keys: ShellKeys::new()
                .bind(KeyBinding::ctrl('r'), ShellAction::App(Cmd::Run))
                .bind(KeyBinding::char('d'), ShellAction::App(Cmd::Delete)),
            layer: None,
            menu: false,
            capture: false,
            interrupts: false,
            dirty: false,
            chrome: false,
            page_takes: vec![],
            list: None,
            esc_top: false,
            took: vec![],
        }
    }

    fn key(&mut self, key: Key) -> Took {
        self.took.clear();
        KeyRouter::route(self, &key);
        self.took.last().cloned().unwrap_or(Took::Nothing)
    }
}

impl App for Host {
    type Status = ();
    fn shell(&self) -> &Shell {
        &self.shell
    }
    fn shell_mut(&mut self) -> &mut Shell {
        &mut self.shell
    }
    fn theme(&self) -> Theme {
        Theme::junie()
    }
    fn too_small(&self) -> TooSmall<'_> {
        TooSmall::new("demo").min(20, 3)
    }
    fn now_ms(&self) -> u64 {
        0
    }
    fn tick(&self) -> u64 {
        0
    }
    fn tick_interval(&self) -> Duration {
        Duration::from_millis(100)
    }
    fn draw(&mut self, _area: Rect, _buf: &mut Buffer, _ctx: &mut RenderCtx) {}
    fn on_input(&mut self, _input: crate::core::event::Input) -> Outcome {
        Outcome::Ignored
    }
}

impl KeyHost for Host {
    type Cmd = Cmd;
    fn shell_keys(&self) -> &ShellKeys<Cmd> {
        &self.keys
    }
    fn layer_key(&mut self, _key: &Key) -> Option<Outcome> {
        let takes = self.layer?;
        if takes {
            self.took.push(Took::Layer);
            return Some(Outcome::Changed);
        }
        Some(Outcome::Ignored)
    }
    fn menu_key(&mut self, _key: &Key) -> Option<Outcome> {
        self.menu.then(|| {
            self.took.push(Took::Menu);
            Outcome::Changed
        })
    }
    fn capturing(&self) -> bool {
        self.capture
    }
    fn interrupt(&mut self) -> Outcome {
        if self.interrupts {
            self.took.push(Took::Interrupt);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }
    fn dirty(&mut self) -> bool {
        self.dirty
    }
    fn shell_action(&mut self, action: ShellAction<Cmd>, _key: &Key) -> Outcome {
        self.took.push(Took::Action(action));
        Outcome::Changed
    }
    fn chrome_key(&mut self, _key: &Key) -> Outcome {
        if self.chrome {
            self.took.push(Took::Chrome);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }
    fn page_key(&mut self, key: &Key) -> Outcome {
        if self.page_takes.contains(key) {
            self.took.push(Took::Page);
            return Outcome::Changed;
        }
        if let Some(l) = self.list.as_mut() {
            let out = l.on_key(key);
            if out == Outcome::Changed {
                self.took.push(Took::Page);
            }
            return out;
        }
        Outcome::Ignored
    }
    fn layer_paste(&mut self, text: &str) -> Option<Outcome> {
        self.layer?;
        self.took.push(Took::Pasted(format!("layer:{text}")));
        Some(Outcome::Ignored)
    }
    fn page_paste(&mut self, text: &str) -> Outcome {
        self.took.push(Took::Pasted(format!("page:{text}")));
        Outcome::Changed
    }
    fn esc_top(&mut self) -> Outcome {
        if self.esc_top {
            self.took.push(Took::EscTop);
            return Outcome::Changed;
        }
        Outcome::Ignored
    }
    fn request_quit(&mut self) -> Outcome {
        self.took.push(Took::Quit);
        Outcome::Changed
    }
    fn note(&mut self, text: &str) {
        self.took.push(Took::Note(text.into()));
    }
}

fn k(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn ch(c: char) -> Key {
    k(KeyCode::Char(c))
}

fn ctrl(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        mods: KeyModifiers::CONTROL,
    }
}

fn with_mods(code: KeyCode, mods: KeyModifiers) -> Key {
    Key { code, mods }
}

#[test]
fn the_key_order_table() {
    type Setup = fn(&mut Host);
    let rows: Vec<(&str, Setup, Key, Took)> = vec![
        (
            "too small: q quits",
            |h| h.shell.too_small = true,
            ch('q'),
            Took::Quit,
        ),
        (
            "too small: Ctrl+C quits",
            |h| h.shell.too_small = true,
            ctrl('c'),
            Took::Quit,
        ),
        (
            "too small: other keys are inert",
            |h| h.shell.too_small = true,
            ch('d'),
            Took::Nothing,
        ),
        (
            "layer before shell keys",
            |h| h.layer = Some(true),
            ctrl('r'),
            Took::Layer,
        ),
        (
            "layer before page",
            |h| {
                h.layer = Some(true);
                h.page_takes = vec![ch('x')]
            },
            ch('x'),
            Took::Layer,
        ),
        (
            "a layer's untaken Ctrl+Q still quits",
            |h| h.layer = Some(false),
            ctrl('q'),
            Took::Quit,
        ),
        (
            "a layer's untaken letter never reaches the page",
            |h| {
                h.layer = Some(false);
                h.page_takes = vec![ch('x')]
            },
            ch('x'),
            Took::Nothing,
        ),
        (
            "open menu before shell keys",
            |h| h.menu = true,
            k(KeyCode::Tab),
            Took::Menu,
        ),
        (
            "editing field takes a letter shortcut",
            |h| {
                h.capture = true;
                h.page_takes = vec![ch('d')]
            },
            ch('d'),
            Took::Page,
        ),
        (
            "editing field passes: shell letter stays quiet",
            |h| h.capture = true,
            ch('d'),
            Took::Nothing,
        ),
        (
            "editing field passes a chord on to the shell",
            |h| h.capture = true,
            ctrl('r'),
            Took::Action(ShellAction::App(Cmd::Run)),
        ),
        (
            "editing field binding the chord keeps it",
            |h| {
                h.capture = true;
                h.page_takes = vec![ctrl('r')]
            },
            ctrl('r'),
            Took::Page,
        ),
        (
            "a priority chord acts before an editing field",
            |h| {
                h.capture = true;
                h.page_takes = vec![ctrl('t')];
                h.keys
                    .priority
                    .push((KeyBinding::ctrl('t'), ShellAction::App(Cmd::Run)));
            },
            ctrl('t'),
            Took::Action(ShellAction::App(Cmd::Run)),
        ),
        (
            "a priority letter stays quiet over an editing field",
            |h| {
                h.capture = true;
                h.page_takes = vec![ch('z')];
                h.keys
                    .priority
                    .push((KeyBinding::char('z'), ShellAction::App(Cmd::Run)));
            },
            ch('z'),
            Took::Page,
        ),
        (
            "an idle-only chord stays quiet over an editing field",
            |h| {
                h.capture = true;
                h.keys.idle_only.push(ShellAction::App(Cmd::Run));
            },
            ctrl('r'),
            Took::Nothing,
        ),
        (
            "editing field: Ctrl+Q quits",
            |h| h.capture = true,
            ctrl('q'),
            Took::Quit,
        ),
        (
            "editing field: q types, never quits",
            |h| h.capture = true,
            ch('q'),
            Took::Nothing,
        ),
        (
            "Ctrl+C: the page refuses first",
            |h| h.interrupts = true,
            ctrl('c'),
            Took::Interrupt,
        ),
        (
            "Ctrl+C: nothing to cancel quits",
            |_| {},
            ctrl('c'),
            Took::Quit,
        ),
        (
            "Ctrl+C over the focused control's selection: it copies (SI17)",
            |h| {
                let id = WidgetId::of("selected");
                h.shell.ring.register(id);
                h.shell.ring.mark_selection(id);
                h.shell.focus.set(Some(id));
                h.interrupts = true;
                h.page_takes = vec![ctrl('c')];
            },
            ctrl('c'),
            Took::Page,
        ),
        (
            "Ctrl+C: a selection off the focused control does not count",
            |h| {
                let (sel, focused) = (WidgetId::of("selected"), WidgetId::of("focused"));
                h.shell.ring.register(sel);
                h.shell.ring.register(focused);
                h.shell.ring.mark_selection(sel);
                h.shell.focus.set(Some(focused));
                h.page_takes = vec![ctrl('c')];
            },
            ctrl('c'),
            Took::Quit,
        ),
        (
            "app chord before the page",
            |h| h.page_takes = vec![ctrl('r')],
            ctrl('r'),
            Took::Action(ShellAction::App(Cmd::Run)),
        ),
        (
            "F1 help",
            |_| {},
            k(KeyCode::F(1)),
            Took::Action(ShellAction::Help),
        ),
        (
            "F10 menu bar",
            |_| {},
            k(KeyCode::F(10)),
            Took::Action(ShellAction::MenuBar),
        ),
        (
            "Ctrl+F2 menu bar (SI10)",
            |_| {},
            with_mods(KeyCode::F(2), KeyModifiers::CONTROL),
            Took::Action(ShellAction::MenuBar),
        ),
        (
            "Ctrl+PgDn next page (SI23)",
            |_| {},
            with_mods(KeyCode::PageDown, KeyModifiers::CONTROL),
            Took::Action(ShellAction::NextPage),
        ),
        (
            "Ctrl+PgUp previous page (SI23)",
            |_| {},
            with_mods(KeyCode::PageUp, KeyModifiers::CONTROL),
            Took::Action(ShellAction::PrevPage),
        ),
        (
            "focused chrome before the page",
            |h| {
                h.chrome = true;
                h.page_takes = vec![k(KeyCode::Enter)]
            },
            k(KeyCode::Enter),
            Took::Chrome,
        ),
        (
            "page",
            |h| h.page_takes = vec![ch('x')],
            ch('x'),
            Took::Page,
        ),
        (
            "page before the fallback ?",
            |h| h.page_takes = vec![ch('?')],
            ch('?'),
            Took::Page,
        ),
        (
            "? help when the page passes",
            |_| {},
            ch('?'),
            Took::Action(ShellAction::Help),
        ),
        ("q quits when clean", |_| {}, ch('q'), Took::Quit),
        (
            "q over unsaved work: a note",
            |h| h.dirty = true,
            ch('q'),
            Took::Note("Unsaved changes · Ctrl+Q asks before quitting".into()),
        ),
        (
            "Esc: the page's ladder",
            |h| h.esc_top = true,
            k(KeyCode::Esc),
            Took::EscTop,
        ),
        (
            "Esc at the top never quits",
            |_| {},
            k(KeyCode::Esc),
            Took::Note("Esc never quits · Ctrl+Q quits".into()),
        ),
        (
            "Esc quits when configured",
            |h| h.keys.quit.esc_quits = true,
            k(KeyCode::Esc),
            Took::Quit,
        ),
        ("Tab moves focus", |_| {}, k(KeyCode::Tab), Took::Nothing),
    ];
    for (name, setup, key, want) in rows {
        let mut h = Host::new();
        setup(&mut h);
        assert_eq!(h.key(key), want, "{name}");
    }
}

fn list_host() -> Host {
    let items = ["alpha", "delta", "dune", "echo"]
        .iter()
        .map(|s| ListItem::new(s))
        .collect();
    let mut h = Host::new();
    h.list = Some(ListBox::new(WidgetId::of("list"), items, SelectMode::Single).type_ahead(true));
    h
}

#[test]
fn a_letter_shortcut_wins_over_a_focused_lists_type_ahead() {
    let mut h = list_host();
    assert_eq!(
        h.key(ch('d')),
        Took::Action(ShellAction::App(Cmd::Delete)),
        "the shortcut takes `d`"
    );
    assert_eq!(
        h.list.as_ref().unwrap().cursor,
        0,
        "and the list did not type it"
    );
    assert_eq!(
        h.key(ch('e')),
        Took::Page,
        "a letter no shortcut takes types"
    );
    assert_eq!(h.list.as_ref().unwrap().cursor, 3);
}

#[test]
fn tab_and_shift_tab_walk_the_ring_after_the_page() {
    let mut h = Host::new();
    let mut ring = crate::core::focus::FocusRing::default();
    ring.register(WidgetId::of("a"));
    ring.register(WidgetId::of("b"));
    h.shell.ring = ring;
    h.key(k(KeyCode::Tab));
    assert_eq!(h.shell.focus.current(), Some(WidgetId::of("a")));
    h.key(k(KeyCode::Tab));
    assert_eq!(h.shell.focus.current(), Some(WidgetId::of("b")));
    h.key(with_mods(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(h.shell.focus.current(), Some(WidgetId::of("a")));
}

#[test]
fn an_activation_that_changed_something_flashes_the_focused_control() {
    let mut h = Host::new();
    h.page_takes = vec![k(KeyCode::Enter)];
    h.shell.focus.set(Some(WidgetId::of("ok")));
    h.key(k(KeyCode::Enter));
    assert_eq!(h.shell.pointer.flashing(0), Some(WidgetId::of("ok")));
}

#[test]
fn a_paste_goes_to_the_top_layer_else_the_page() {
    let paste = |h: &mut Host| {
        h.took.clear();
        let out = KeyRouter::input(h, crate::core::event::Input::Paste("hi".into()));
        (out, h.took.clone())
    };
    let mut h = Host::new();
    assert_eq!(
        paste(&mut h),
        (Outcome::Changed, vec![Took::Pasted("page:hi".into())])
    );
    h.layer = Some(false);
    assert_eq!(
        paste(&mut h),
        (Outcome::Consumed, vec![Took::Pasted("layer:hi".into())]),
        "a layer that takes no text still keeps it from the page"
    );
    h.layer = None;
    h.shell.too_small = true;
    assert_eq!(paste(&mut h), (Outcome::Consumed, vec![]));
}

#[test]
fn the_help_section_lists_the_live_shell_keys() {
    let keys = ShellKeys::<u8>::new().bind(KeyBinding::ctrl('r'), ShellAction::App(1));
    let s = keys.help_section("Shell", |c| (c == 1).then_some("Run"));
    let row = |action: &str| {
        s.rows
            .iter()
            .find(|(_, a)| a == action)
            .map(|(k, _)| k.clone())
    };
    assert_eq!(row("This reference").as_deref(), Some("F1 ?"));
    assert_eq!(row("Run").as_deref(), Some("Ctrl+R"));
    assert_eq!(row("Quit").as_deref(), Some("Ctrl+Q q"));
    assert_eq!(row("Next page").as_deref(), Some("Ctrl+PgDn"));

    assert_eq!(row("Previous control").as_deref(), Some("Shift+Tab"));
    let unbound = ShellKeys::<u8>::new().unbind(KeyBinding::char('?'));
    let s = unbound.help_section("Shell", |_| None);
    assert_eq!(
        s.rows
            .iter()
            .find(|(_, a)| a == "This reference")
            .map(|(k, _)| k.as_str()),
        Some("F1"),
        "an unbound key is not listed"
    );
}
