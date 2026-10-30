use ratatui_core::{buffer::Buffer, layout::Rect};
use termrock_layout::{Constraints, Size};
use termrock_runtime::author::composition_leaf;
use termrock_runtime::{App, Cx, Response, Runtime, Ui};
use termrock_shell::AppShell;
use termrock_theme::Theme;

struct Probe;
impl App for Probe {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let one = composition_leaf(
            |_: &Ui<'_>, _: Constraints| Size {
                min: (1, 1),
                preferred: (1, 1),
            },
            |_: &mut Ui<'_>, _: Rect| {},
        );
        let shell = AppShell::new();
        let slots = shell.slots(&one, &one, &one);
        let measured = slots.measure(ui, Constraints::loose(20, 10)).unwrap();
        assert_eq!(
            measured.min.1, 5,
            "three one-row children and two required chrome gaps need five rows"
        );
    }
}
#[test]
fn shell_minimum_includes_its_required_gaps() {
    let area = Rect::new(0, 0, 20, 10);
    let mut runtime = Runtime::new(Probe, Theme::junie());
    let _ = runtime.initialize();
    runtime
        .draw_buffer(area, &mut Buffer::empty(area))
        .commit_presented();
}

#[test]
fn shell_minimum_reserves_configured_strips_and_clamps_tiny_constraints() {
    // The default one-row leaves cannot shrink the configured four and two
    // row strip reservations; two one-row gaps and one body row are needed.
    struct ConfiguredProbe;
    impl App for ConfiguredProbe {
        fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
            Response::ignored()
        }
        fn draw(&self, ui: &mut Ui<'_>) {
            let one = composition_leaf(
                |_: &Ui<'_>, _: Constraints| Size {
                    min: (1, 1),
                    preferred: (1, 1),
                },
                |_: &mut Ui<'_>, _: Rect| {},
            );
            let slots = AppShell::new()
                .chrome(termrock_shell::Chrome::new().header(4).footer(2).gap(1))
                .slots(&one, &one, &one);
            assert_eq!(
                slots.measure(ui, Constraints::loose(20, 10)).unwrap().min.1,
                9
            );
            assert_eq!(
                slots.measure(ui, Constraints::loose(20, 4)).unwrap().min.1,
                4
            );

            let omitted = AppShell::new()
                .chrome(termrock_shell::Chrome::new().header(0).footer(0).gap(3))
                .slots(&one, &one, &one);
            assert_eq!(
                omitted
                    .measure(ui, Constraints::loose(20, 10))
                    .unwrap()
                    .min
                    .1,
                1
            );

            let inset = AppShell::new()
                .chrome(termrock_shell::Chrome::new().inset(3))
                .slots(&one, &one, &one);
            assert_eq!(
                inset.measure(ui, Constraints::loose(5, 10)).unwrap().min.0,
                5
            );
            assert_eq!(
                inset.measure(ui, Constraints::loose(4, 10)).unwrap().min.0,
                4
            );
        }
    }
    let area = Rect::new(0, 0, 20, 10);
    let mut runtime = Runtime::new(ConfiguredProbe, Theme::junie());
    let _ = runtime.initialize();
    runtime
        .draw_buffer(area, &mut Buffer::empty(area))
        .commit_presented();
}

const BODY: termrock_runtime::Id = termrock_runtime::Id::root("review.shell.body");
struct NestedProbe;
impl App for NestedProbe {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let one = composition_leaf(
            |_: &Ui<'_>, _: Constraints| Size {
                min: (1, 1),
                preferred: (1, 1),
            },
            |_: &mut Ui<'_>, _: Rect| {},
        );
        let body = composition_leaf(
            |_: &Ui<'_>, _: Constraints| Size {
                min: (1, 1),
                preferred: (1, 1),
            },
            |ui: &mut Ui<'_>, area: Rect| {
                ui.register_control(BODY, area, termrock_runtime::Focusability::Focusable);
            },
        );
        let sibling = composition_leaf(
            |_: &Ui<'_>, _: Constraints| Size {
                min: (0, 0),
                preferred: (1, 7),
            },
            |_: &mut Ui<'_>, _: Rect| {},
        );
        let shell = AppShell::new();
        let outer = termrock_runtime::composition::CompositionSlots::column([
            termrock_runtime::composition::ContentSlot::nested(
                shell.slots(&one, &body, &one),
                termrock_layout::ChildPolicy::default(),
            ),
            termrock_runtime::composition::ContentSlot::new(
                &sibling,
                termrock_layout::ChildPolicy::default(),
            ),
        ]);
        outer.draw(ui).unwrap();
    }
}
#[test]
fn nested_shell_keeps_minimum_body_when_sibling_can_shrink() {
    let area = Rect::new(0, 0, 20, 10);
    let mut runtime = Runtime::new(NestedProbe, Theme::junie());
    let _ = runtime.initialize();
    runtime
        .draw_buffer(area, &mut Buffer::empty(area))
        .commit_presented();
    assert!(
        runtime.area_of(BODY).is_some(),
        "shell minimum body vanished although sibling could shrink from seven to five rows"
    );
}
