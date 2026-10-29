//! Executable consumer showing borrowed component composition through the
//! public Termrock facade.

use core::fmt;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use termrock::{
    App, Button, Cx, Id, ItemKey, Response, Runtime, Tabs, TabsState, Theme, composition,
    layout::{Composition, Insets},
};

const RUN: Id = Id::root("example.run");
const WORKSPACES: Id = Id::root("example.workspaces");

#[derive(Clone, Copy)]
struct Workspace {
    key: ItemKey,
    label: &'static str,
}

impl fmt::Display for Workspace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label)
    }
}

fn workspace_key(workspace: &Workspace) -> ItemKey {
    workspace.key
}

type WorkspaceTabs = Tabs<'static, Workspace, fn(&Workspace) -> ItemKey>;

struct Workbench {
    run: Button<'static>,
    tabs: WorkspaceTabs,
    tabs_state: TabsState,
    workspaces: [Workspace; 2],
}

impl Workbench {
    fn new() -> Self {
        Self {
            run: Button::new(RUN, "Run"),
            tabs: Tabs::new(WORKSPACES).key(workspace_key),
            tabs_state: TabsState::default(),
            workspaces: [
                Workspace {
                    key: ItemKey::text("source"),
                    label: "源泉 🚀",
                },
                Workspace {
                    key: ItemKey::text("review"),
                    label: "Long workspace · 👩🏽‍💻 — Unicode text beyond the preferred tab width",
                },
            ],
        }
    }
}

impl App for Workbench {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.run.update(cx).erase()
            | self
                .tabs
                .update(cx, &mut self.tabs_state, &self.workspaces)
                .erase()
    }

    fn draw(&self, ui: &mut termrock::Ui<'_>) {
        composition::Group::row([
            composition::button(&self.run).max(Some(8), None),
            composition::Group::column([
                composition::Group::wrap([composition::tabs(
                    &self.tabs,
                    &self.tabs_state,
                    &self.workspaces,
                )
                .flex(1)
                .max(Some(24), None)])
                .into_child()
                .flex(1),
                composition::Group::wrap(std::iter::empty()).into_child(),
            ])
            .into_child()
            .flex(1),
        ])
        .spec(Composition::new().gap(1).padding(Insets::all(1)))
        .show(ui)
        .expect("composition is within the supported depth");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let area = termrock::Rect::new(0, 0, 48, 10);
    let mut runtime = Runtime::new(Workbench::new(), Theme::junie());
    let prepared = runtime.prepare(area)?;
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height))?;
    terminal.draw(|frame| {
        runtime
            .draw_prepared(prepared, frame)
            .expect("prepared area matches the framework terminal");
    })?;
    runtime.acknowledge_presented(prepared)?;
    Ok(())
}
