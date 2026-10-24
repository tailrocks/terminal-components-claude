//! External-consumer coverage for composition ownership and shell adapters.
use std::cell::Cell;
use std::rc::Rc;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_core::{Id, Input, KeyModifiers, Mouse, MouseKind, Response};
use termrock_layout::{ChildPolicy, Composition, Constraints, Insets, Size};
use termrock_runtime::author::{CompositionLeaf, Measure, composition_leaf};
use termrock_runtime::composition::{CompositionSlots, ContentSlot};
use termrock_runtime::{App, Cx, Focusability, Runtime, Ui};
use termrock_shell::{AppShell, AppShellSlots, Chrome};
use termrock_theme::Theme;

const HEADER: Id = Id::root("shell.interop.header");
const BODY: Id = Id::root("shell.interop.body");
const FOOTER: Id = Id::root("shell.interop.footer");
const BORROWED: Id = Id::root("shell.interop.borrowed");
const OWNED: Id = Id::root("shell.interop.owned");

fn fixed(id: Id) -> impl CompositionLeaf {
    composition_leaf(
        |_: &Ui<'_>, constraints: Constraints| Size {
            min: (1, 1),
            preferred: constraints.max,
        },
        move |ui: &mut Ui<'_>, area: Rect| {
            ui.register_control(id, area, Focusability::Focusable);
            ui.set_bar_cursor(id, Position::new(area.x, area.y));
        },
    )
}

struct ShellProbe {
    chrome: Chrome,
    padding: u16,
}

impl App for ShellProbe {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let header = fixed(HEADER);
        let body = fixed(BODY);
        let footer = fixed(FOOTER);
        let shell = AppShell::new().chrome(self.chrome);
        // This exact assignment is the pre-existing public contract.
        let slots: CompositionSlots<'_> = shell.slots(&header, &body, &footer);
        let slots = slots.spec(Composition::new().padding(Insets::all(self.padding)));
        let measured = slots
            .measure(ui, Constraints::loose(ui.full().width, ui.full().height))
            .unwrap();
        assert!(measured.min.1 <= ui.full().height);
        slots.draw(ui).unwrap();
    }
}

fn present<A: App>(runtime: &mut Runtime<A>, area: Rect) {
    for _ in 0..8 {
        if runtime.needs_settle() {
            let _ = runtime.settle();
        }
        runtime
            .draw_buffer(area, &mut Buffer::empty(area))
            .commit_presented();
        if !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    assert!(!runtime.needs_present(), "shell did not settle");
}

fn send<A: App>(runtime: &mut Runtime<A>, area: Rect, input: Input) {
    let mut next = Some(input);
    for _ in 0..8 {
        if let Some(input) = next.take()
            && let Err(pending) = runtime.handle(input)
        {
            next = Some(pending.into_input());
        }
        present(runtime, area);
        if next.is_none() && !runtime.needs_settle() && !runtime.needs_present() {
            return;
        }
    }
    assert!(next.is_none(), "runtime did not admit input");
}

#[test]
fn public_shell_slots_keep_composition_type_spec_and_resolved_regions() {
    let root = Rect::new(10, 5, 40, 12);
    let chrome = Chrome::new().header(2).footer(1).gap(1).inset(2);
    let padding = 1;
    let allocated = Rect::new(11, 6, 38, 10);
    let expected = chrome.regions(allocated);
    let mut runtime = Runtime::new(ShellProbe { chrome, padding }, Theme::junie());
    let _ = runtime.initialize();
    present(&mut runtime, root);

    for (id, area) in [
        (HEADER, expected.header),
        (BODY, expected.body),
        (FOOTER, expected.footer),
    ] {
        assert_eq!(runtime.area_of(id), Some(area));
        runtime.set_focus(Some(id));
        present(&mut runtime, root);
        assert_eq!(runtime.cursor(), Some(Position::new(area.x, area.y)));
        assert!(runtime.cursor_bar());
        send(
            &mut runtime,
            root,
            Input::Mouse(Mouse {
                kind: MouseKind::Move,
                pos: Position::new(area.x + area.width / 2, area.y + area.height / 2),
                mods: KeyModifiers::NONE,
            }),
        );
        assert_eq!(runtime.hover(), Some(id));
    }
}

struct TextLeaf<'a> {
    text: &'a str,
    id: Id,
    drops: Option<Rc<Cell<usize>>>,
}

impl Measure for TextLeaf<'_> {
    fn measure(&self, _: &Ui<'_>, constraints: Constraints) -> Size {
        Size {
            min: (1, 1),
            preferred: (
                (self.text.len() as u16).min(constraints.max.0),
                1.min(constraints.max.1),
            ),
        }
    }
}

impl CompositionLeaf for TextLeaf<'_> {
    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        ui.register_control(self.id, area, Focusability::Focusable);
        ui.set_bar_cursor(self.id, Position::new(area.x, area.y));
    }
}

impl Drop for TextLeaf<'_> {
    fn drop(&mut self) {
        if let Some(drops) = &self.drops {
            drops.set(drops.get() + 1);
        }
    }
}

struct OwnedProbe<'a> {
    borrowed_text: &'a str,
    drops: Rc<Cell<usize>>,
}

impl App for OwnedProbe<'_> {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let label_header = TextLeaf {
            text: self.borrowed_text,
            id: HEADER,
            drops: None,
        };
        let label_body = TextLeaf {
            text: self.borrowed_text,
            id: BODY,
            drops: None,
        };
        let label_footer = TextLeaf {
            text: self.borrowed_text,
            id: FOOTER,
            drops: None,
        };
        // The shell-owned adapter keeps the old component-data lifetime.
        let shell = AppShell::new().slots(&label_header, &label_body, &label_footer);
        let _ = shell
            .measure(ui, Constraints::loose(ui.full().width, ui.full().height))
            .unwrap();
        shell.draw(ui).unwrap();

        let borrowed = TextLeaf {
            text: self.borrowed_text,
            id: BORROWED,
            drops: None,
        };
        let children = CompositionSlots::row([
            ContentSlot::new(&borrowed, ChildPolicy::default().flex(1)),
            ContentSlot::owned(
                TextLeaf {
                    text: "domain value",
                    id: OWNED,
                    drops: Some(Rc::clone(&self.drops)),
                },
                ChildPolicy::default().flex(1),
            ),
        ]);
        let slots = CompositionSlots::column([ContentSlot::nested(
            children,
            ChildPolicy::default().flex(1),
        )]);
        slots.draw(ui).unwrap();
    }
}

#[test]
fn owned_and_borrowed_leaves_share_runtime_bounds_and_drop_once() {
    let text = String::from("borrowed label");
    let drops = Rc::new(Cell::new(0));
    let area = Rect::new(0, 0, 40, 10);
    let mut runtime = Runtime::new(
        OwnedProbe {
            borrowed_text: &text,
            drops: Rc::clone(&drops),
        },
        Theme::junie(),
    );
    let _ = runtime.initialize();
    runtime
        .draw_buffer(area, &mut Buffer::empty(area))
        .commit_presented();
    assert!(runtime.area_of(BORROWED).is_some());
    assert!(runtime.area_of(OWNED).is_some());
    assert!(runtime.area_of(HEADER).is_some());
    assert!(runtime.area_of(BODY).is_some());
    assert!(runtime.area_of(FOOTER).is_some());
    assert_eq!(drops.get(), 1, "the owned leaf is dropped exactly once");
    runtime.set_focus(Some(OWNED));
    runtime
        .draw_buffer(area, &mut Buffer::empty(area))
        .commit_presented();
    assert_eq!(
        drops.get(),
        2,
        "the second frame owns and drops one new leaf"
    );
    assert_eq!(runtime.focus(), Some(OWNED));
    assert_eq!(
        runtime.cursor(),
        runtime
            .area_of(OWNED)
            .map(|area| Position::new(area.x, area.y))
    );
    let owned_area = runtime.area_of(OWNED).unwrap();
    send(
        &mut runtime,
        area,
        Input::Mouse(Mouse {
            kind: MouseKind::Move,
            pos: Position::new(owned_area.x, owned_area.y),
            mods: KeyModifiers::NONE,
        }),
    );
    assert_eq!(runtime.hover(), Some(OWNED));
}

#[test]
fn app_shell_builder_slots_keep_composition_interop() {
    let leaf = fixed(BODY);
    let group: CompositionSlots<'_> = AppShell::new().with_slots().body(&leaf).slots();
    let nested = ContentSlot::nested(
        group.spec(Composition::new().gap(2)),
        ChildPolicy::default(),
    );
    let _outer = CompositionSlots::column([nested]);
    let direct_nested = ContentSlot::nested(
        AppShell::new().slots(&leaf, &leaf, &leaf),
        ChildPolicy::default().flex(1),
    );
    let _direct_outer = CompositionSlots::column([direct_nested]);
    let _: Option<AppShellSlots<'static>> = None;
}
