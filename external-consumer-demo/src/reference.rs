//! A real external consumer of borrowed reference adapters.
//!
//! This small screen delegates rendering to native Button and TextInput
//! components. The adapters borrow those configurations and the TextInput
//! states from the application; the ReferenceGrid owns only the temporary
//! table description.

use termrock::controls::Button;
use termrock::fields::{TextInput, TextInputState};
use termrock::{
    App, Cx, Id, ReferenceCase, ReferenceGrid, ReferenceSpecimen, ReferenceState, ReferenceTarget,
    Response, Status, Ui,
};

/// The native button rendered by the example column.
pub const BUTTON_ID: Id = Id::root("reference-gallery.button");
/// The native text input rendered by the example column.
pub const INPUT_ID: Id = Id::root("reference-gallery.input");

/// A borrowed native button adapter used by the external reference sample.
pub struct ButtonReference<'a> {
    rest: &'a Button<'static>,
    selected: &'a Button<'static>,
    disabled: &'a Button<'static>,
    busy: &'a Button<'static>,
}

impl<'a> ButtonReference<'a> {
    /// Borrow the component configurations used by the authored cases.
    pub const fn new(
        rest: &'a Button<'static>,
        selected: &'a Button<'static>,
        disabled: &'a Button<'static>,
        busy: &'a Button<'static>,
    ) -> Self {
        Self {
            rest,
            selected,
            disabled,
            busy,
        }
    }
}

impl ReferenceSpecimen for ButtonReference<'_> {
    fn supports(&self, case: ReferenceCase) -> bool {
        matches!(
            case,
            ReferenceCase::Rest
                | ReferenceCase::Hover
                | ReferenceCase::Pressed
                | ReferenceCase::Focused
                | ReferenceCase::FocusedHover
                | ReferenceCase::Selected
                | ReferenceCase::Disabled
                | ReferenceCase::Busy
        )
    }

    fn target(&self, case: ReferenceCase) -> Option<ReferenceTarget> {
        target_for(BUTTON_ID, case)
    }

    fn draw(&self, case: ReferenceCase, ui: &mut Ui<'_>, area: termrock::author::Rect) {
        match case {
            ReferenceCase::Selected => self.selected.draw(ui, area),
            ReferenceCase::Disabled => self.disabled.draw(ui, area),
            ReferenceCase::Busy => self.busy.draw(ui, area),
            _ => self.rest.draw(ui, area),
        };
    }
}

/// A borrowed native text-input adapter used by the external sample.
pub struct TextInputReference<'a> {
    rest: &'a TextInput<'static>,
    disabled: &'a TextInput<'static>,
    busy: &'a TextInput<'static>,
    error: &'a TextInput<'static>,
    idle_state: &'a TextInputState,
    editing_state: &'a TextInputState,
}

impl<'a> TextInputReference<'a> {
    /// Borrow component configurations and caller-owned editor states.
    pub const fn new(
        rest: &'a TextInput<'static>,
        disabled: &'a TextInput<'static>,
        busy: &'a TextInput<'static>,
        error: &'a TextInput<'static>,
        idle_state: &'a TextInputState,
        editing_state: &'a TextInputState,
    ) -> Self {
        Self {
            rest,
            disabled,
            busy,
            error,
            idle_state,
            editing_state,
        }
    }
}

impl ReferenceSpecimen for TextInputReference<'_> {
    fn supports(&self, case: ReferenceCase) -> bool {
        matches!(
            case,
            ReferenceCase::Rest
                | ReferenceCase::Hover
                | ReferenceCase::Pressed
                | ReferenceCase::Focused
                | ReferenceCase::FocusedHover
                | ReferenceCase::Disabled
                | ReferenceCase::Busy
                | ReferenceCase::Error
                | ReferenceCase::Editing
        )
    }

    fn target(&self, case: ReferenceCase) -> Option<ReferenceTarget> {
        target_for(INPUT_ID, case)
    }

    fn draw(&self, case: ReferenceCase, ui: &mut Ui<'_>, area: termrock::author::Rect) {
        let state = if case == ReferenceCase::Editing {
            self.editing_state
        } else {
            self.idle_state
        };
        match case {
            ReferenceCase::Disabled => self.disabled.draw(ui, area, state),
            ReferenceCase::Busy => self.busy.draw(ui, area, state),
            ReferenceCase::Error => self.error.draw(ui, area, state),
            _ => self.rest.draw(ui, area, state),
        };
    }
}

fn target_for(id: Id, case: ReferenceCase) -> Option<ReferenceTarget> {
    let state = match case {
        ReferenceCase::Hover => ReferenceState::HOVERED,
        ReferenceCase::Pressed => ReferenceState::HOVERED | ReferenceState::PRESSED,
        ReferenceCase::Focused => ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE,
        ReferenceCase::FocusedHover => {
            ReferenceState::FOCUSED | ReferenceState::FOCUS_VISIBLE | ReferenceState::HOVERED
        }
        ReferenceCase::Rest
        | ReferenceCase::Selected
        | ReferenceCase::Disabled
        | ReferenceCase::Busy
        | ReferenceCase::Error
        | ReferenceCase::Editing
        | ReferenceCase::Empty
        | ReferenceCase::Loading
        | ReferenceCase::Overflow
        | ReferenceCase::Narrow => return None,
    };
    Some(ReferenceTarget::new(id, state))
}

/// A screen that demonstrates the public borrowed reference API externally.
pub struct ReferenceGallery {
    button_rest: Button<'static>,
    button_selected: Button<'static>,
    button_disabled: Button<'static>,
    button_busy: Button<'static>,
    input_rest: TextInput<'static>,
    input_disabled: TextInput<'static>,
    input_busy: TextInput<'static>,
    input_error: TextInput<'static>,
    input_idle_state: TextInputState,
    input_editing_state: TextInputState,
}

impl ReferenceGallery {
    /// Build the native component configurations shown by the sample.
    pub fn new() -> Self {
        let button_rest = Button::new(BUTTON_ID, "Run");
        let button_selected = Button::new(BUTTON_ID, "Run").checked(true);
        let button_disabled = Button::new(BUTTON_ID, "Run").disabled(true);
        let button_busy = Button::new(BUTTON_ID, "Run").status(Status::Busy);

        let input_rest = TextInput::new(INPUT_ID).value("recipe");
        let input_disabled = TextInput::new(INPUT_ID).value("recipe").disabled(true);
        let input_busy = TextInput::new(INPUT_ID)
            .value("recipe")
            .status(Status::Busy);
        let input_error = TextInput::new(INPUT_ID)
            .value("recipe")
            .status(Status::Error);
        let input_idle_state = TextInputState::default();
        let mut input_editing_state = TextInputState::default();
        input_editing_state.begin("recipe");
        let _ = input_editing_state.place_cursor(3, 12);

        Self {
            button_rest,
            button_selected,
            button_disabled,
            button_busy,
            input_rest,
            input_disabled,
            input_busy,
            input_error,
            input_idle_state,
            input_editing_state,
        }
    }

    /// Borrow the Button configurations used by this sample.
    pub fn button_adapter(&self) -> ButtonReference<'_> {
        ButtonReference::new(
            &self.button_rest,
            &self.button_selected,
            &self.button_disabled,
            &self.button_busy,
        )
    }

    /// Build the borrowed two-column grid.
    pub fn grid(&self) -> ReferenceGrid<'_> {
        ReferenceGrid::new()
            .rows(&ReferenceCase::ALL)
            .column("Button", self.button_adapter())
            .column(
                "Text input",
                TextInputReference::new(
                    &self.input_rest,
                    &self.input_disabled,
                    &self.input_busy,
                    &self.input_error,
                    &self.input_idle_state,
                    &self.input_editing_state,
                ),
            )
    }
}

impl Default for ReferenceGallery {
    fn default() -> Self {
        Self::new()
    }
}

impl App for ReferenceGallery {
    fn update(&mut self, _cx: &mut Cx<'_>) -> Response<()> {
        Response::ignored()
    }

    fn draw(&self, ui: &mut Ui<'_>) {
        let area = ui.full();
        let _ = self.grid().draw(ui, area);
    }
}
