//! Active placeholders change paint only; canonical editing and cursor stay shared.
use junie_tui::{
    App, Cx, Id, KeyCode, Rect, Response, TextInput, TextInputState, Theme, Ui, UpdateCause,
};
use junie_tui_testing::Harness;
const INPUT: Id = Id::root("placeholder.input");
struct Page {
    editor: TextInputState,
    value: String,
    active: bool,
    show: bool,
    width: u16,
}
impl App for Page {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        if cx.update_cause() == UpdateCause::Bootstrap {
            if self.active {
                self.editor.begin("");
            }
            cx.focus(INPUT);
        }
        TextInput::new(INPUT)
            .read_only(!self.active)
            .update(cx, &mut self.editor, &mut self.value)
            .erase()
    }
    fn draw(&self, ui: &mut Ui<'_>) {
        TextInput::new(INPUT)
            .value(&self.value)
            .read_only(!self.active)
            .placeholder("検索…")
            .placeholder_while_editing(self.show)
            .draw(ui, Rect::new(0, 0, self.width, 1), &self.editor);
    }
}
fn page(active: bool, show: bool, width: u16) -> Harness<Page> {
    Harness::new(
        Page {
            editor: TextInputState::default(),
            value: String::new(),
            active,
            show,
            width,
        },
        Theme::junie(),
        24,
        3,
    )
}
#[test]
fn active_placeholder_replaces_and_restores_without_moving_ownership() {
    let mut h = page(true, true, 24);
    assert!(h.text().contains("検索…"));
    let cursor = h.cursor();
    assert!(cursor.is_some());
    assert_eq!(h.focus(), Some(INPUT));
    let _ = h.key(KeyCode::Char('界'));
    assert!(!h.text().contains("検索"));
    assert!(h.text().contains('界'));
    assert_eq!(h.app().editor.draft_text(), Some("界"));
    let _ = h.key(KeyCode::Backspace);
    assert!(h.text().contains("検索…"));
    assert_eq!(h.cursor(), cursor);
    assert_eq!(h.focus(), Some(INPUT));
    assert_eq!(h.app().editor.draft_text(), Some(""));
    assert!(h.diagnostics().is_empty(), "{:?}", h.diagnostics());
}
#[test]
fn default_active_is_blank_and_idle_placeholder_unchanged() {
    let hidden = page(true, false, 24);
    let shown = page(true, true, 24);
    assert!(!hidden.text().contains("検索"));
    assert_eq!(hidden.cursor(), shown.cursor());
    assert_eq!(hidden.focus(), shown.focus());
    let idle_default = page(false, false, 24);
    let idle_opt_in = page(false, true, 24);
    assert_eq!(idle_default.text(), idle_opt_in.text());
    assert!(idle_default.text().contains("検索…"));
}
#[test]
fn unicode_placeholder_tiny_widths_preserve_cursor_and_clip() {
    for width in 0..12 {
        let hidden = page(true, false, width);
        let shown = page(true, true, width);
        assert_eq!(hidden.cursor(), shown.cursor(), "width {width}");
        assert_eq!(hidden.focus(), shown.focus(), "width {width}");
        assert_eq!(hidden.diagnostics(), shown.diagnostics(), "width {width}");
        if width < 5 {
            assert!(!shown.text().contains('検'), "width {width}");
        }
    }
}
