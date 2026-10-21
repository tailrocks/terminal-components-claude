//! Executable consumer exercising the shell crate through the public
//! Termrock APIs only: the [`termrock_shell::AppShell`] frame, a
//! [`termrock_shell::Router`] of pages, and the [`termrock_shell::KeyRouter`]
//! key tiers, with unrelated domain data (a file-browser trail).

use termrock::{
    App, Cx, Family, Focusability, FrameRead, Id, Intent, ItemKey, Key, KeyCode, KeyModifiers,
    Rect, Response, RowUi, Runtime, Theme, Ui,
    author::{CompositionLeaf, Constraints, Measure, Size},
};
use termrock_shell::{
    AppShell, Chrome, Footer, Hint, KeyHost, KeyRouter, Page, Router, ShellAction, ShellKeys,
};

const BODY: Id = Id::root("example.body");

/// One routed screen: a labelled row naming its trail.
struct Browse {
    label: String,
}

impl Page for Browse {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_control(BODY, ui.full(), Focusability::Focusable);
        let mut r = RowUi::new(
            ui,
            BODY,
            Family::LIST,
            Variant::DEFAULT,
            ui.state(BODY),
            ItemKey::index(0),
            ui.full(),
        );
        r.gutter();
        r.label(&format!("{} — press o to open deeper", self.label));
    }

    fn label(&self) -> String {
        self.label.clone()
    }
}

use termrock::Variant;

struct Workbench {
    router: Router<Browse>,
    keys: ShellKeys<Cmd>,
    footer: Footer,
    note: Option<String>,
    quit: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Cmd {
    Open,
    About,
}

impl Workbench {
    fn new() -> Self {
        let mut router: Router<Browse> = Router::new(Browse {
            label: "Disk".into(),
        });
        router.push(Browse {
            label: "Usage".into(),
        });
        Self {
            router,
            keys: ShellKeys::new()
                .bind(
                    KeyCode::Char('o'),
                    KeyModifiers::NONE,
                    ShellAction::App(Cmd::Open),
                )
                .bind(
                    KeyCode::Char('a'),
                    KeyModifiers::NONE,
                    ShellAction::App(Cmd::About),
                ),
            footer: Footer::new().tab_next(true),
            note: None,
            quit: false,
        }
    }
}

impl KeyHost for Workbench {
    type Cmd = Cmd;

    fn shell_keys(&self) -> &ShellKeys<Cmd> {
        &self.keys
    }

    fn shell_action(&mut self, action: ShellAction<Cmd>, _key: &Key) -> Response<()> {
        match action {
            ShellAction::App(Cmd::Open) => {
                self.router.push(Browse {
                    label: format!("Folder {}", self.router.depth()),
                });
                Response::changed()
            }
            ShellAction::App(Cmd::About) => {
                self.note = Some("shell_consumer · AppShell + Router + KeyRouter".into());
                Response::changed()
            }
            ShellAction::Help => {
                self.note = Some("F1 help is the app's to show".into());
                Response::changed()
            }
            _ => Response::ignored(),
        }
    }

    fn page_key(&mut self, _key: &Key) -> Response<()> {
        Response::ignored()
    }

    fn request_quit(&mut self) -> Response<()> {
        // A clean app leaves at once; an app with stakes hands the leave to
        // its LeaveGuard instead (see termrock_shell::LeaveGuard).
        self.quit = true;
        Response::changed()
    }

    fn note(&mut self, text: &str) {
        self.note = Some(text.to_owned());
    }
}

impl App for Workbench {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut r = Response::ignored();
        for intent in cx.intents(BODY) {
            if let Intent::Key(key) = intent {
                r |= KeyRouter::route(self, &key);
            }
        }
        if self.quit {
            cx.quit();
        }
        r
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let header = Header {
            trail: self.router.crumb(Id::root("example.crumb"), |p| p.label()),
        };
        let footer = FooterBar {
            hints: self.footer.compose(),
            note: self.note.clone(),
        };
        let body = Body {
            page: self.router.current(),
        };
        let _ = AppShell::new()
            .chrome(Chrome::new().gap(1))
            .show(ui, &header, &body, &footer);
    }
}

struct Header {
    trail: termrock_shell::Crumb,
}

impl Measure for Header {
    fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (0, 1),
            preferred: (c.max.0, 1),
        }
    }
}

impl CompositionLeaf for Header {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        let id = self.trail.id;
        ui.register_control(id, logical, Focusability::ClickOnly);
        let mut r = RowUi::new(
            ui,
            id,
            Family::LIST,
            Variant::DEFAULT,
            ui.state(id),
            ItemKey::text("trail"),
            logical,
        );
        r.label(&self.trail.segments.join(" › "));
    }
}

struct Body<'a> {
    page: Option<&'a Browse>,
}

impl Measure for Body<'_> {
    fn measure(&self, _ui: &Ui<'_>, c: Constraints) -> Size {
        Size {
            min: (0, 0),
            preferred: c.max,
        }
    }
}

impl CompositionLeaf for Body<'_> {
    fn draw(&self, ui: &mut Ui<'_>, _logical: Rect) {
        if let Some(page) = self.page {
            page.draw(ui);
        }
    }
}

struct FooterBar {
    hints: Vec<Hint>,
    note: Option<String>,
}

impl Measure for FooterBar {
    fn measure(&self, _ui: &Ui<'_>, _c: Constraints) -> Size {
        Size {
            min: (0, 1),
            preferred: (u16::MAX, 1),
        }
    }
}

impl CompositionLeaf for FooterBar {
    fn draw(&self, ui: &mut Ui<'_>, logical: Rect) {
        let text = match &self.note {
            Some(n) => n.clone(),
            None => self
                .hints
                .iter()
                .map(|h| format!("{} {}", h.key, h.action))
                .collect::<Vec<_>>()
                .join("  "),
        };
        let id = Id::root("example.footer");
        ui.register_control(id, logical, Focusability::ClickOnly);
        let mut r = RowUi::new(
            ui,
            id,
            Family::LIST,
            Variant::DEFAULT,
            ui.state(id),
            ItemKey::text("footer"),
            logical,
        );
        r.label(&text);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let area = Rect::new(0, 0, 48, 10);
    let mut runtime = Runtime::new(Workbench::new(), Theme::junie());
    let prepared = runtime.prepare(area)?;
    let mut terminal =
        ratatui::Terminal::new(ratatui::backend::TestBackend::new(area.width, area.height))?;
    terminal.draw(|frame| {
        runtime
            .draw_prepared(prepared, frame)
            .expect("prepared area matches the framework terminal");
    })?;
    runtime.acknowledge_presented(prepared)?;
    Ok(())
}
