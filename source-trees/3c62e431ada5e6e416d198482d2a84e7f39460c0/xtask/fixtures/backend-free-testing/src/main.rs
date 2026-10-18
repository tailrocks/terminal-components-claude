use junie_tui::{
    App, Chord, Cx, Input, Key, KeyCode, KeyModifiers, MediaKeyCode, ModifierKeyCode, Response,
    Runtime, Theme, Ui,
};

#[derive(Default)]
struct Consumer {
    updates: usize,
}
impl App for Consumer {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        self.updates += 1;
        Response::ignored()
    }
    fn draw(&self, _ui: &mut Ui<'_>) {}
}

fn main() {
    let media = Key {
        code: KeyCode::Media(MediaKeyCode::PlayPause),
        mods: KeyModifiers::SUPER,
    };
    assert_eq!(media.chord(), Chord::with(media.code, media.mods));
    let modifier = Input::Key(Key {
        code: KeyCode::Modifier(ModifierKeyCode::IsoLevel5Shift),
        mods: KeyModifiers::HYPER,
    });
    assert!(matches!(modifier, Input::Key(_)));
    let _runtime = Runtime::new(Consumer::default(), Theme::junie());
    let _harness = junie_tui_testing::Harness::new(Consumer::default(), Theme::junie(), 20, 5);
}
