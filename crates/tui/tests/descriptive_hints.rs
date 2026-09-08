//! Descriptive keycaps share normal hint geometry without registering input.
use junie_tui::{
    App, Chord, Cx, Hint, HintBar, HintKey, HintLayer, Id, KeyCode, KeyHint, KeyModifiers, Rect,
    Response, Theme, Ui,
};
use junie_tui_testing::Harness;
const ID: Id = Id::root("descriptive.hints");
struct ChordPage {
    chord: Chord,
    from_hint: bool,
}
impl App for ChordPage {
    fn update(&mut self, _: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        let hint = Hint {
            key: HintKey::Chord(self.chord),
            label: "Run",
            priority: 50,
        };
        let props = if self.from_hint {
            KeyHint::from_hint(ID, &hint)
        } else {
            KeyHint::new(ID, self.chord, "Run")
        };
        props.draw(ui, ui.full());
    }
}
#[test]
fn chord_wrapper_preserves_every_existing_cell_and_style() {
    for chord in [
        Chord::key(KeyCode::Enter),
        Chord::key(KeyCode::Left),
        Chord::with(KeyCode::Char('s'), KeyModifiers::CONTROL),
        Chord::with(
            KeyCode::Backspace,
            KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::SHIFT,
        ),
    ] {
        for width in [1, 4, 20, 60] {
            let old = Harness::new(
                ChordPage {
                    chord,
                    from_hint: false,
                },
                Theme::junie(),
                width,
                1,
            );
            let wrapped = Harness::new(
                ChordPage {
                    chord,
                    from_hint: true,
                },
                Theme::junie(),
                width,
                1,
            );
            assert_eq!(old.buffer(), wrapped.buffer());
            assert!(wrapped.diagnostics().is_empty());
        }
    }
}
struct MixedPage {
    layer: HintLayer,
    deliveries: usize,
}
impl App for MixedPage {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.deliveries = self
            .deliveries
            .saturating_add(cx.intents(ID).count())
            .saturating_add(usize::from(cx.command().is_some()));
        Response::ignored()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        HintBar::new(ID, &self.layer).draw(ui, Rect::new(0, 0, ui.full().width, 1));
        let key = KeyHint::descriptive(Id::root("descriptive.single"), "Type", "Filter");
        assert_eq!(key.width(), 11);
        key.draw(ui, Rect::new(0, 1, ui.full().width, 1));
    }
}
fn mixed(width: u16) -> Harness<MixedPage> {
    Harness::new(
        MixedPage {
            layer: HintLayer {
                hints: vec![
                    Hint {
                        key: HintKey::Label("Type"),
                        label: "Filter",
                        priority: 90,
                    },
                    Hint {
                        key: HintKey::Chord(Chord::key(KeyCode::Enter)),
                        label: "Run",
                        priority: 80,
                    },
                ],
                ..HintLayer::empty()
            },
            deliveries: 0,
        },
        Theme::junie(),
        width,
        2,
    )
}
#[test]
fn mixed_labels_paint_and_never_publish_focus_or_binding_owners() {
    let mut h = mixed(50);
    assert!(h.text().contains("Type Filter"));
    assert!(h.text().contains("Enter Run"));
    assert_eq!(h.cell(0, 1).symbol(), "T");
    assert_eq!(h.cell(4, 1).symbol(), " ");
    assert_eq!(h.cell(5, 1).symbol(), "F");
    assert_eq!(h.focus(), None);
    assert_eq!(h.area_of(ID), None);
    for key in [KeyCode::Char('T'), KeyCode::Enter, KeyCode::Tab] {
        let _ = h.key(key);
    }
    assert_eq!(h.app().deliveries, 0);
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn descriptive_labels_clip_through_the_same_hint_path() {
    for width in 0..=24 {
        let h = mixed(width);
        assert_eq!(h.buffer().area.width, width);
        assert_eq!(h.focus(), None);
        assert!(
            h.diagnostics().is_empty(),
            "width={width}: {:?}",
            h.diagnostics()
        );
    }
}
