use ratatui_core::buffer::Buffer;
use ratatui_core::layout::{Position, Rect};
use termrock_core::{Id, Input, KeyModifiers, Mouse, MouseKind, Response};
use termrock_layout::{Constraints, Size};
use termrock_runtime::author::composition_leaf;
use termrock_runtime::{App, Cx, Focusability, Runtime, Ui};
use termrock_shell::{AppShell, Chrome, Page, Stakes, page_leaf};
use termrock_theme::Theme;

const HEADER: Id = Id::root("shell.geometry.header");
const BODY: Id = Id::root("shell.geometry.body");
const FOOTER: Id = Id::root("shell.geometry.footer");

struct PageProbe;

impl Page for PageProbe {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, _ui: &mut Ui<'_>) {}

    fn draw_in(&self, ui: &mut Ui<'_>, body: Rect) {
        ui.register_control(BODY, body, Focusability::Focusable);
        ui.set_bar_cursor(BODY, Position::new(body.x, body.y));
    }

    fn label(&self) -> String {
        String::from("probe")
    }

    fn stakes(&self) -> Stakes {
        Stakes::Clean
    }
}

struct ShellProbe {
    chrome: Chrome,
}

impl App for ShellProbe {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let header = composition_leaf(
            |_: &Ui<'_>, constraints: Constraints| Size {
                min: (0, 1),
                preferred: (constraints.max.0, 1),
            },
            |ui: &mut Ui<'_>, area: Rect| {
                ui.register_control(HEADER, area, Focusability::Focusable);
                ui.set_bar_cursor(HEADER, Position::new(area.x, area.y));
            },
        );
        let footer = composition_leaf(
            |_: &Ui<'_>, constraints: Constraints| Size {
                min: (0, 1),
                preferred: (constraints.max.0, 1),
            },
            |ui: &mut Ui<'_>, area: Rect| {
                ui.register_control(FOOTER, area, Focusability::Focusable);
                ui.set_bar_cursor(FOOTER, Position::new(area.x, area.y));
            },
        );
        let page = PageProbe;
        let body = page_leaf(&page);
        let shell = AppShell::new().chrome(self.chrome);
        let slots = shell.slots(&header, &body, &footer);
        let measured = slots
            .measure(ui, Constraints::loose(ui.full().width, ui.full().height))
            .unwrap();
        assert!(
            measured.min.1 <= ui.full().height,
            "a filling page must not claim the reserved shell height as mandatory"
        );
        let _ = shell.show(ui, &header, &body, &footer);
    }
}

fn present(runtime: &mut Runtime<ShellProbe>, area: Rect) {
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
    assert!(!runtime.needs_present(), "shell probe did not settle");
}

fn send(runtime: &mut Runtime<ShellProbe>, area: Rect, input: Input) {
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
    assert!(next.is_none(), "shell probe input was not admitted");
}

fn point(area: Rect) -> Position {
    Position::new(area.x + area.width / 2, area.y + area.height / 2)
}

#[test]
fn shell_regions_match_registered_hit_bounds_and_focused_cursors() {
    let root = Rect::new(10, 5, 40, 12);
    let chrome = Chrome::new().header(2).footer(1).gap(1).inset(2);
    let expected = chrome.regions(root);
    assert_eq!(expected.header, Rect::new(10, 5, 40, 2));
    assert_eq!(expected.body, Rect::new(12, 8, 36, 7));
    assert_eq!(expected.footer, Rect::new(10, 16, 40, 1));

    let mut runtime = Runtime::new(ShellProbe { chrome }, Theme::junie());
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
        assert_eq!(runtime.focus(), Some(id));
        assert_eq!(runtime.cursor(), Some(Position::new(area.x, area.y)));
        assert!(runtime.cursor_bar());

        send(
            &mut runtime,
            root,
            Input::Mouse(Mouse {
                kind: MouseKind::Move,
                pos: point(area),
                mods: KeyModifiers::NONE,
            }),
        );
        assert_eq!(runtime.hover(), Some(id));
    }
}

#[test]
fn shell_omitted_strips_tiny_roots_and_zero_regions_share_the_resolver() {
    let root = Rect::new(7, 3, 1, 2);
    let chrome = Chrome::new().header(0).footer(1).gap(2).inset(4);
    let expected = chrome.regions(root);
    assert!(expected.header.is_empty());
    assert_eq!(expected.body, Rect::new(7, 3, 0, 0));
    assert_eq!(expected.footer, Rect::new(7, 4, 1, 1));

    let mut runtime = Runtime::new(ShellProbe { chrome }, Theme::junie());
    let _ = runtime.initialize();
    present(&mut runtime, root);
    assert_eq!(runtime.area_of(HEADER), None);
    assert_eq!(runtime.area_of(BODY), None);
    assert_eq!(runtime.area_of(FOOTER), Some(expected.footer));

    let zero = Rect::new(7, 3, 0, 0);
    let expected_zero = chrome.regions(zero);
    runtime
        .draw_buffer(zero, &mut Buffer::empty(zero))
        .commit_presented();
    assert!(runtime.area_of(HEADER).is_none());
    assert!(runtime.area_of(BODY).is_none());
    assert!(runtime.area_of(FOOTER).is_none());
    assert!(expected_zero.body.is_empty());

    let compact_root = Rect::new(7, 3, 10, 2);
    let compact_chrome = Chrome::new().gap(1);
    let compact_expected = compact_chrome.regions(compact_root);
    assert_eq!(compact_expected.header, Rect::new(7, 3, 10, 1));
    assert_eq!(compact_expected.body, Rect::new(7, 5, 10, 0));
    assert_eq!(compact_expected.footer, Rect::new(7, 4, 10, 1));

    let mut compact = Runtime::new(
        ShellProbe {
            chrome: compact_chrome,
        },
        Theme::junie(),
    );
    let _ = compact.initialize();
    present(&mut compact, compact_root);
    assert_eq!(compact.area_of(HEADER), Some(compact_expected.header));
    assert_eq!(compact.area_of(BODY), None);
    assert_eq!(compact.area_of(FOOTER), Some(compact_expected.footer));
}
