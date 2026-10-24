use termrock_core::event::{Input, KeyCode};
use termrock_core::keys::KeyModifiers;
use termrock_core::response::Response;

use super::*;
use crate::quit::QuitKeys;

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
    keys: ShellKeys<Cmd>,
    layer: Option<bool>,
    layer_response: Option<Response<()>>,
    menu: bool,
    capture: bool,
    interrupts: bool,
    selection: bool,
    dirty: bool,
    chrome: bool,
    page_takes: Vec<Key>,
    esc_top: bool,
    took: Vec<Took>,
}

impl Host {
    fn new() -> Self {
        Self {
            keys: ShellKeys::new()
                .bind(
                    KeyCode::Char('r'),
                    KeyModifiers::CONTROL,
                    ShellAction::App(Cmd::Run),
                )
                .bind(
                    KeyCode::Char('d'),
                    KeyModifiers::NONE,
                    ShellAction::App(Cmd::Delete),
                ),
            layer: None,
            layer_response: None,
            menu: false,
            capture: false,
            interrupts: false,
            selection: false,
            dirty: false,
            chrome: false,
            page_takes: vec![],
            esc_top: false,
            took: vec![],
        }
    }

    fn key(&mut self, key: Key) -> Took {
        self.took.clear();
        let _ = KeyRouter::route(self, &key);
        self.took.last().cloned().unwrap_or(Took::Nothing)
    }

    fn flow(&mut self, key: Key) -> (bool, bool) {
        self.took.clear();
        let out = KeyRouter::route(self, &key);
        (out.is_consumed(), out.is_changed())
    }
}

impl KeyHost for Host {
    type Cmd = Cmd;

    fn shell_keys(&self) -> &ShellKeys<Cmd> {
        &self.keys
    }

    fn layer_key(&mut self, _key: &Key) -> Option<Response<()>> {
        let takes = self.layer?;
        if takes {
            self.took.push(Took::Layer);
            return Some(Response::changed());
        }
        Some(self.layer_response.clone().unwrap_or_default())
    }

    fn menu_key(&mut self, _key: &Key) -> Option<Response<()>> {
        self.menu.then(|| {
            self.took.push(Took::Menu);
            Response::changed()
        })
    }

    fn capturing(&self) -> bool {
        self.capture
    }

    fn interrupt(&mut self) -> Response<()> {
        if self.interrupts {
            self.took.push(Took::Interrupt);
            return Response::changed();
        }
        Response::ignored()
    }

    fn dirty(&mut self) -> bool {
        self.dirty
    }

    fn has_selection(&self) -> bool {
        self.selection
    }

    fn shell_action(&mut self, action: ShellAction<Cmd>, _key: &Key) -> Response<()> {
        self.took.push(Took::Action(action));
        Response::changed()
    }

    fn chrome_key(&mut self, _key: &Key) -> Response<()> {
        if self.chrome {
            self.took.push(Took::Chrome);
            return Response::changed();
        }
        Response::ignored()
    }

    fn page_key(&mut self, key: &Key) -> Response<()> {
        if self.page_takes.contains(key) {
            self.took.push(Took::Page);
            return Response::changed();
        }
        Response::ignored()
    }

    fn layer_paste(&mut self, _text: &str) -> Option<Response<()>> {
        self.layer?;
        self.took.push(Took::Pasted("layer:hi".into()));
        Some(Response::ignored())
    }

    fn page_paste(&mut self, _text: &str) -> Response<()> {
        self.took.push(Took::Pasted("page:hi".into()));
        Response::changed()
    }

    fn esc_top(&mut self) -> Response<()> {
        if self.esc_top {
            self.took.push(Took::EscTop);
            return Response::changed();
        }
        Response::ignored()
    }

    fn request_quit(&mut self) -> Response<()> {
        self.took.push(Took::Quit);
        Response::changed()
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
    // The PR #26 too-small rows (q/Ctrl+C quit, other keys inert) are
    // runtime-owned on parity: the runtime gates input behind its own
    // too-small screen before any app code runs.
    let rows: Vec<(&str, Setup, Key, Took)> = vec![
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
            k(KeyCode::F(1)),
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
                h.keys.priority.push(KeyBinding {
                    chord: chord(KeyCode::Char('t'), KeyModifiers::CONTROL),
                    action: ShellAction::App(Cmd::Run),
                    priority: 5,
                });
            },
            ctrl('t'),
            Took::Action(ShellAction::App(Cmd::Run)),
        ),
        (
            "a priority letter stays quiet over an editing field",
            |h| {
                h.capture = true;
                h.page_takes = vec![ch('z')];
                h.keys.priority.push(KeyBinding {
                    chord: chord(KeyCode::Char('z'), KeyModifiers::NONE),
                    action: ShellAction::App(Cmd::Run),
                    priority: 5,
                });
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
                h.selection = true;
                h.interrupts = true;
                h.page_takes = vec![ctrl('c')];
            },
            ctrl('c'),
            Took::Page,
        ),
        (
            "Ctrl+C without a selection does not copy",
            |h| h.page_takes = vec![ctrl('c')],
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
    ];
    for (name, setup, key, want) in rows {
        let mut h = Host::new();
        setup(&mut h);
        assert_eq!(h.key(key), want, "{name}");
    }
}

// On parity the runtime serves Tab and BackTab out of its own focus ring
// before the router runs, so the PR #26 ring-walking tests are runtime
// coverage; the shell keeps the bindings only as vocabulary.
#[test]
fn tab_reaches_the_fallback_tier_when_the_runtime_passes_it() {
    let mut h = Host::new();
    assert_eq!(h.key(k(KeyCode::Tab)), Took::Action(ShellAction::FocusNext));
    assert_eq!(
        h.key(with_mods(KeyCode::BackTab, KeyModifiers::SHIFT)),
        Took::Action(ShellAction::FocusPrev)
    );
}

#[test]
fn a_paste_goes_to_the_top_layer_else_the_page() {
    let paste = |h: &mut Host| {
        h.took.clear();
        let out = KeyRouter::input(h, &Input::Paste("hi".into()));
        (out.is_changed(), h.took.clone())
    };
    let mut h = Host::new();
    assert_eq!(paste(&mut h), (true, vec![Took::Pasted("page:hi".into())]));
    h.layer = Some(false);
    assert_eq!(
        paste(&mut h),
        (false, vec![Took::Pasted("layer:hi".into())]),
        "a layer that takes no text still keeps it from the page"
    );
}

#[test]
fn a_layer_that_passes_still_consumes_the_key() {
    let mut h = Host::new();
    h.layer = Some(false);
    let (consumed, changed) = h.flow(ch('x'));
    assert!(
        consumed && !changed,
        "the layer keeps the key from the page"
    );
}

#[test]
fn a_passing_layers_invalidate_payload_survives_the_consumed_fold() {
    let mut h = Host::new();
    h.layer = Some(false);
    h.layer_response = Some(Response::ignored().repaint());
    let out = KeyRouter::route(&mut h, &ch('x'));
    assert!(out.is_consumed(), "the layer keeps the key from the page");
    assert!(
        out.is_changed(),
        "the layer's repaint payload is not discarded"
    );
}

#[test]
fn unbind_removes_a_key_from_every_tier() {
    let keys = ShellKeys::<u8>::new()
        .bind_priority(
            KeyCode::Char('g'),
            KeyModifiers::CONTROL,
            ShellAction::App(1),
        )
        .unbind(KeyCode::Char('g'), KeyModifiers::CONTROL);
    let g = Key {
        code: KeyCode::Char('g'),
        mods: KeyModifiers::CONTROL,
    };
    assert!(lookup(&keys.priority, &g).is_none());
    assert!(lookup(&keys.keys, &g).is_none());
    assert!(lookup(&keys.fallback, &g).is_none());
    let f1 = Key {
        code: KeyCode::F(1),
        mods: KeyModifiers::NONE,
    };
    assert!(lookup(&keys.keys, &f1).is_some(), "other keys stay bound");
}

#[test]
fn the_quit_letter_note_honours_the_configured_letter() {
    let mut h = Host::new();
    h.keys.quit = QuitKeys::DEFAULT.letter(Some('x'));
    h.dirty = true;
    assert_eq!(
        h.key(ch('x')),
        Took::Note("Unsaved changes · Ctrl+Q asks before quitting".into())
    );
    h.keys.quit = QuitKeys::DEFAULT.letter(None);
    h.page_takes = vec![ch('x')];
    assert_eq!(
        h.key(ch('x')),
        Took::Page,
        "an unbound letter reaches the page"
    );
}
