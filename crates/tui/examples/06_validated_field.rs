//! `COMPONENT_ARCHITECTURE.md` §17 example 6, verbatim (crate name is temporary: `tui_next` → `junie_tui` at Slice 5).
#![expect(dead_code, reason = "verbatim from §17 example 6")]

use tui_next::{
    BlurPolicy, Cx, Field, FieldError, Id, Rect, Response, TextAction, TextInput, TextInputState,
    Ui, id,
};

const EMAIL: Id = id!("email");

struct Form {
    email: String,                // the controlled value — the caller owns it
    email_st: TextInputState,     // durable interaction state only
    server_error: Option<String>, // async result from the application
}

fn valid_email(s: &str) -> Result<(), FieldError> {
    if s.contains('@') {
        Ok(())
    } else {
        Err(FieldError {
            message: "Enter a valid address".into(),
            code: Some("email"),
        })
    }
}

fn check_uniqueness(_s: &str) -> Option<String> {
    None
} // stands in for the application effect

/// The one constructor for this control, used by both phases (§13 "props are built once").
/// It takes no `&self`, so `update` can still pass `&mut self.email` alongside it.
fn email_input() -> TextInput<'static> {
    TextInput::new(EMAIL)
        .validate(&valid_email) // fn item, via the blanket `Validate` impl
        .blur(BlurPolicy::CommitAndValidate)
}

impl Form {
    fn update(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let r = email_input().update(cx, &mut self.email_st, &mut self.email);

        if let Some(TextAction::Committed) = r.action_ref() {
            self.server_error = check_uniqueness(&self.email); // application effect
            self.email_st
                .set_error(self.server_error.as_deref().map(|m| FieldError {
                    message: m.to_owned().into(),
                    code: Some("dup"),
                }));
        }
        r.erase()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        // `Field` is draw-time chrome only: no `Id`, no focus stop, no `update`. The control
        // keeps its identity, so one id is registered per field (§21 item 7).
        Field::new("Email", email_input().value(&self.email))
            .required(true)
            .help("We only use this for sign-in.")
            .error(self.server_error.as_deref())
            .draw(ui, area, &self.email_st);
    }
}

fn main() {}
// `draw` is `&self` and takes `&TextInputState`: committing or validating from draw is a
// compile error, which is what removes the five render-time commits of §1.2(5).
