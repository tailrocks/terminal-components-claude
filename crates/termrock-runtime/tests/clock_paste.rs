//! `Runtime::handle` owns the visual-baseline clock paste.
//! The old `Input::Paste` arm always delivered the body to the typing owner, so the
//! paused `1100` case fails on that arm: the step is not observed and the text is pasted.

use std::ffi::OsString;

use ratatui_core::buffer::Buffer;
use ratatui_core::layout::Rect;
use termrock_core::{Id, Input, Intent};
use termrock_runtime::{
    App, Cx, Focusability, Runtime, TEST_CLOCK_ENV, TEST_CLOCK_PASTE_PREFIX, Ui,
};
use termrock_theme::{StateFlags, Theme};

const EDITOR: Id = Id::root("clock-paste.editor");
const AREA: Rect = Rect::new(0, 0, 40, 8);

struct EnvRestore {
    previous: Option<OsString>,
}

impl EnvRestore {
    fn set(value: Option<&str>) -> Self {
        let previous = std::env::var_os(TEST_CLOCK_ENV);
        match value {
            Some(value) =>
            // SAFETY: nextest runs this integration test in its own process. Drop writes the
            // previous value back before the test returns, and no other thread in this process
            // reads or writes `JUNIE_TUI_TEST_CLOCK`.
            unsafe {
                std::env::set_var(TEST_CLOCK_ENV, value);
            },
            None =>
            // SAFETY: same isolation as `set_var`. Removing the variable is the unset case.
            unsafe {
                std::env::remove_var(TEST_CLOCK_ENV);
            },
        }
        Self { previous }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        match self.previous.take() {
            Some(value) =>
            // SAFETY: restores the value captured before this test changed the variable.
            unsafe {
                std::env::set_var(TEST_CLOCK_ENV, value);
            },
            None =>
            // SAFETY: the variable was absent on entry; remove any value this test set.
            unsafe {
                std::env::remove_var(TEST_CLOCK_ENV);
            },
        }
    }
}

#[derive(Default)]
struct Model {
    paused: bool,
    steps: Vec<u64>,
    pastes: Vec<String>,
}

impl App for Model {
    fn update(&mut self, cx: &mut Cx<'_>) -> termrock_runtime::Response<()> {
        for intent in cx.intents(EDITOR) {
            if let Intent::Paste(text) = intent {
                self.pastes.push(text.to_owned());
            }
        }
        termrock_runtime::Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        ui.register_editor(
            EDITOR,
            Rect::new(0, 0, 10, 1),
            Focusability::Focusable,
            StateFlags::EDITING,
        );
    }

    fn accept_paused_clock_step(&mut self, step_ms: u64) -> bool {
        if !self.paused {
            return false;
        }
        self.steps.push(step_ms);
        true
    }
}

fn present(rt: &mut Runtime<Model>, buf: &mut Buffer) {
    for _ in 0..8 {
        rt.draw_buffer(AREA, buf).commit_presented();
        if !rt.needs_settle() {
            return;
        }
        let _ = rt.settle();
    }
    assert!(
        !rt.needs_settle(),
        "fixture focus did not settle before paste"
    );
}

fn boot(paused: bool) -> Runtime<Model> {
    let mut rt = Runtime::new(
        Model {
            paused,
            ..Model::default()
        },
        Theme::junie(),
    );
    let mut buf = Buffer::empty(AREA);
    let _ = rt.initialize();
    present(&mut rt, &mut buf);
    assert_eq!(rt.typing_owner(), Some(EDITOR));
    rt
}

fn deliver(paused: bool, env: Option<&str>, body: &str) -> (Vec<u64>, Vec<String>) {
    let _env = EnvRestore::set(env);
    let mut rt = boot(paused);
    assert!(
        rt.handle(Input::Paste(body.to_owned())).is_ok(),
        "presented runtime rejected the paste"
    );
    let app = rt.app();
    (app.steps.clone(), app.pastes.clone())
}

fn clock(rest: &str) -> String {
    let mut text = String::from(TEST_CLOCK_PASTE_PREFIX);
    text.push_str(rest);
    text
}

#[test]
fn unset_env_delivers_clock_paste_to_the_typing_owner() {
    let body = clock("100");
    let (steps, pastes) = deliver(true, None, &body);
    assert!(steps.is_empty());
    assert_eq!(pastes, [body]);
}

#[test]
fn enabled_unpaused_app_does_not_consume_the_clock_paste() {
    let body = clock("100");
    let (steps, pastes) = deliver(false, Some("1"), &body);
    assert!(steps.is_empty());
    assert_eq!(pastes, [body]);
}

#[test]
fn enabled_paused_app_consumes_a_positive_clock_step() {
    let (steps, pastes) = deliver(true, Some("1"), &clock("1100"));
    assert_eq!(steps, [1100]);
    assert!(pastes.is_empty());
}

#[test]
fn enabled_paused_app_does_not_consume_zero_or_empty_clock_paste() {
    for rest in ["0", ""] {
        let body = clock(rest);
        let (steps, pastes) = deliver(true, Some("1"), &body);
        assert!(steps.is_empty(), "{rest:?}");
        assert_eq!(pastes, [body], "{rest:?}");
    }
}
