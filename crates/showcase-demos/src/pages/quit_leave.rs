use termrock::author::VerticalWindow;
use termrock::shell::quit::DialogAnswer;
use termrock::shell::{
    KeyContext, Leave, LeaveDialog, LeaveDialogKind, LeaveGuard, LeaveKey, QuitKeys, Resolved,
    Stakes,
};
use termrock::{
    Action, ActionKey, Binding, Button, Chord, Cx, Dialog, DialogAction, DialogState, Focusability,
    FrameRead, Id, Intent, ItemKey, Key, KeyCode, KeyModifiers, Part, Rect, Response, ScrollRegion,
    ScrollState, Select, SelectState, StateFlags, Tabs, TabsState, TextInput, TextInputState, Ui,
    Variant, wrap,
};

use super::{Page, PageStatus, PageUpdate, card_panel_window, frame_with_body_rows};

const PAGE_TABS: Id = Id::root("showcase_ui::pages::quit_leave.page_tabs");
const PLAY_SCROLL: Id = Id::root("showcase_ui::pages::quit_leave.play_scroll");
const OPTIONS_SCROLL: Id = Id::root("showcase_ui::pages::quit_leave.options_scroll");
const STATES_SCROLL: Id = Id::root("showcase_ui::pages::quit_leave.states_scroll");
const TITLE: Id = Id::root("showcase_ui::pages::quit_leave.title");
const LEAVE: Id = Id::root("showcase_ui::pages::quit_leave.leave");
const QUIT: Id = Id::root("showcase_ui::pages::quit_leave.quit");
const EXPORT: Id = Id::root("showcase_ui::pages::quit_leave.export");
const GUARD_DIALOG: Id = Id::root("showcase_ui::pages::quit_leave.guard_dialog");
const OPTION_IDS: [Id; 6] = [
    Id::root("showcase_ui::pages::quit_leave.option.stakes"),
    Id::root("showcase_ui::pages::quit_leave.option.save"),
    Id::root("showcase_ui::pages::quit_leave.option.letter"),
    Id::root("showcase_ui::pages::quit_leave.option.esc_quits"),
    Id::root("showcase_ui::pages::quit_leave.option.ctrl_c"),
    Id::root("showcase_ui::pages::quit_leave.option.ctrl_q"),
];

const CARD_CHROME_ROWS: usize = 3;
const OPTIONS_CONTENT_ROWS: usize = OPTION_IDS.len() * 2 + CARD_CHROME_ROWS;
const QUIT_KEYS_NOTES_START_ROW: usize = 6;

const SECTION_NAMES: &[&str] = &["Playground", "Options", "States"];
const SAVED_TITLE: &str = "Q3 settlement report";
const LABEL_WIDTH: u16 = 12;
const WIDE_LAYOUT: u16 = 96;
const BODY_START_ROW: u16 = 5;

const OPTION_KEYS: [&str; 6] = [
    "quitleave.stakes",
    "quitleave.save",
    "quitleave.letter",
    "quitleave.esc_quits",
    "quitleave.ctrl_c",
    "quitleave.ctrl_q",
];
const OPTION_LABELS: [&str; 6] = ["Stakes", "Save", "Quit letter", "Esc", "Ctrl+C", "Ctrl+Q"];
const OPTION_ITEMS: [&[&str]; 6] = [
    &["clean", "unsaved", "running"],
    &["off", "on"],
    &["q", "x", "none"],
    &["off", "on"],
    &["off", "on"],
    &["off", "on"],
];
const OPTION_HINTS: &[(&str, &str)] = &[("↑ ↓", "Choose"), ("Enter", "Choose")];
const TAB_HINTS: &[(&str, &str)] = &[("← →", "Section"), ("1–3", "Jump")];
const SCROLL_HINTS: &[(&str, &str)] = &[("Wheel", "Scroll"), ("PgUp/Dn", "Scroll")];
const TEXT_HINTS: &[(&str, &str)] = &[
    ("Type", "Edit title"),
    ("Enter", "Commit"),
    ("Esc", "Revert"),
];
const BUTTON_HINTS: &[(&str, &str)] = &[("Enter / Space", "Press"), ("Tab", "Focus next")];
const DIALOG_HINTS: &[(&str, &str)] = &[("← →", "Choose"), ("Enter", "Activate"), ("Esc", "Stay")];

const PAGE_UP: ActionKey = ActionKey::application("showcase.quit_leave.page_up");
const PAGE_DOWN: ActionKey = ActionKey::application("showcase.quit_leave.page_down");

#[derive(Clone, Copy)]
enum ScrollCommand {
    PageUp,
    PageDown,
}

const SCROLL_BINDINGS: &[Binding<ScrollCommand>] = &[
    Binding {
        action: PAGE_UP,
        chord: Some(Chord::key(KeyCode::PageUp)),
        cmd: ScrollCommand::PageUp,
        label: "Page up",
        priority: 70,
        visible: true,
    },
    Binding {
        action: PAGE_DOWN,
        chord: Some(Chord::key(KeyCode::PageDown)),
        cmd: ScrollCommand::PageDown,
        label: "Page down",
        priority: 70,
        visible: true,
    },
];

const SAVE_ACTIONS: [Action<'static>; 3] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::DISCARD, "Discard"),
    Action::new(ActionKey::SAVE, "Save"),
];
const DISCARD_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::DISCARD, "Discard"),
];
const RUNNING_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Stay"),
    Action::danger(ActionKey::CONFIRM, "Stop"),
];

const STATE_NAMES: [&str; 5] = ["Rest", "Hover", "Pressed", "Focused", "Disabled"];
const VARIANT_NAMES: [&str; 4] = ["Subtle", "Danger", "Primary", "Secondary"];
const STATE_ACTIONS: [&str; 4] = ["Cancel", "Discard", "Save", "Stay"];
const VARIANTS: [Variant; 4] = [
    Variant::SUBTLE,
    Variant::DANGER,
    Variant::PRIMARY,
    Variant::SECONDARY,
];

fn option_id(index: usize) -> Id {
    OPTION_IDS[index]
}

fn state_button_id(row: usize, column: usize) -> Id {
    Id::root("showcase_ui::pages::quit_leave.states.button").index(row * 4 + column)
}

fn leave_button() -> Button<'static> {
    Button::new(LEAVE, "Leave for Overview").variant(Variant::SECONDARY)
}

fn quit_button() -> Button<'static> {
    Button::new(QUIT, "Quit").variant(Variant::SECONDARY)
}

fn export_button(running: bool) -> Button<'static> {
    Button::new(EXPORT, if running { "Stop export" } else { "Run export" })
        .variant(Variant::SECONDARY)
}

fn state_button(row: usize, column: usize) -> Button<'static> {
    let button = Button::new(state_button_id(row, column), STATE_ACTIONS[column])
        .variant(VARIANTS[column])
        .disabled(row == 4);
    if row == 3 && column == 0 {
        button.autofocus()
    } else {
        button
    }
}

fn option_name(index: usize, value: usize) -> &'static str {
    OPTION_ITEMS[index][value.min(OPTION_ITEMS[index].len().saturating_sub(1))]
}

fn page_option_index(key: &str) -> Option<usize> {
    OPTION_KEYS.iter().position(|candidate| *candidate == key)
}

fn plain_key(code: KeyCode) -> Key {
    Key {
        code,
        mods: KeyModifiers::NONE,
    }
}

fn ctrl_key(character: char) -> Key {
    Key {
        code: KeyCode::Char(character),
        mods: KeyModifiers::CONTROL,
    }
}

fn contexts() -> [(&'static str, KeyContext); 4] {
    [
        ("clean", KeyContext::default()),
        (
            "text focus",
            KeyContext {
                text_focus: true,
                ..KeyContext::default()
            },
        ),
        (
            "dirty",
            KeyContext {
                dirty: true,
                ..KeyContext::default()
            },
        ),
        (
            "selection",
            KeyContext {
                selection: true,
                ..KeyContext::default()
            },
        ),
    ]
}

fn scroll_window(view: Rect, content_len: usize, state: &ScrollState) -> Option<VerticalWindow> {
    if view.is_empty() || content_len == 0 {
        return None;
    }
    let first = ScrollRegion::view(state, view, content_len)
        .offset()
        .min(content_len);
    let visible_rows = usize::from(view.height).min(content_len.saturating_sub(first));
    if visible_rows == 0 {
        return None;
    }
    let visible = Rect {
        height: u16::try_from(visible_rows).ok()?,
        ..view
    };
    VerticalWindow::new(visible, content_len, first).ok()
}

fn nested_window(
    parent: VerticalWindow,
    source_start: usize,
    source_rows: usize,
    x: u16,
    width: u16,
) -> Option<VerticalWindow> {
    if source_rows == 0 || width == 0 {
        return None;
    }
    let visible = parent.visible_source_range();
    let start = visible.start.max(source_start);
    let end = visible.end.min(source_start.saturating_add(source_rows));
    if start >= end {
        return None;
    }
    let y = parent.physical_row_for_source(start)?;
    let height = u16::try_from(end.saturating_sub(start)).ok()?;
    VerticalWindow::new(
        Rect::new(x, y, width, height),
        source_rows,
        start.saturating_sub(source_start),
    )
    .ok()
}

fn register_scroll_keys(ui: &mut Ui<'_>, id: Id) {
    ui.publish_bindings(id, ui.state(id), SCROLL_BINDINGS);
}

fn update_scroll_keys(cx: &mut Cx<'_>, id: Id, state: &mut ScrollState) -> Response<()> {
    let mut response = Response::ignored();
    for intent in cx.intents(id) {
        let Intent::Binding(action) = intent else {
            continue;
        };
        match Binding::command(SCROLL_BINDINGS, action) {
            Some(ScrollCommand::PageUp) => state.page_up(),
            Some(ScrollCommand::PageDown) => state.page_down(),
            None => continue,
        }
        response |= Response::changed().for_id(id);
    }
    response
}

fn window_row(window: VerticalWindow, row: usize) -> Option<u16> {
    window.physical_row_for_source(row)
}

fn heading_style(ui: &mut Ui<'_>) -> termrock::author::PaintStyle {
    ui.style(
        termrock::Family::PANEL,
        Variant::DEFAULT,
        Part::TITLE,
        StateFlags::empty(),
    )
    .style
}

fn text_style(ui: &mut Ui<'_>) -> termrock::author::PaintStyle {
    ui.style(
        termrock::Family::LIST,
        Variant::DEFAULT,
        Part::META,
        StateFlags::empty(),
    )
    .style
}

fn key_hint_style(ui: &mut Ui<'_>) -> termrock::author::PaintStyle {
    ui.style(
        termrock::Family::HELP,
        Variant::DEFAULT,
        Part::KEY,
        StateFlags::empty(),
    )
    .style
}

fn stakes_label(stakes: &Stakes) -> String {
    match stakes {
        Stakes::Clean => "Clean".to_owned(),
        Stakes::Unsaved { changes } => format!("Unsaved · {changes} change(s)"),
        Stakes::Running { what } => format!("Running · {what}"),
    }
}

fn leave_label(to: &Leave<&'static str>) -> String {
    match to {
        Leave::Quit => "Leave::Quit".to_owned(),
        Leave::To(route) => format!("Leave::To({route})"),
    }
}

fn go_label(to: &Leave<&'static str>) -> String {
    match to {
        Leave::Quit => "quit".to_owned(),
        Leave::To(route) => format!("go to {route}"),
    }
}

#[derive(Clone, Copy)]
struct PlaygroundLayout {
    wide: bool,
    document_height: usize,
    keys_height: usize,
    content_height: usize,
}

#[derive(Clone, Copy)]
struct StatesLayout {
    columns: usize,
    header_rows: usize,
    content_height: usize,
}

pub struct QuitLeavePage {
    page_tabs: TabsState,
    options: [usize; 6],
    option_states: [SelectState; 6],
    play_scroll: ScrollState,
    options_scroll: ScrollState,
    states_scroll: ScrollState,
    title: String,
    saved: String,
    title_state: TextInputState,
    running: bool,
    guard: LeaveGuard<&'static str>,
    guard_dialog: Option<LeaveDialog>,
    guard_state: DialogState,
    last_request: String,
    last_answer: String,
    last_sample: String,
}

impl QuitLeavePage {
    pub fn new() -> Self {
        Self {
            page_tabs: TabsState::default(),
            options: [0, 1, 0, 0, 1, 1],
            option_states: std::array::from_fn(|_| SelectState::default()),
            play_scroll: ScrollState::default(),
            options_scroll: ScrollState::default(),
            states_scroll: ScrollState::default(),
            title: SAVED_TITLE.to_owned(),
            saved: SAVED_TITLE.to_owned(),
            title_state: TextInputState::default(),
            running: false,
            guard: LeaveGuard::new(GUARD_DIALOG),
            guard_dialog: None,
            guard_state: DialogState::default(),
            last_request: "—".to_owned(),
            last_answer: "—".to_owned(),
            last_sample: "No state sample activated".to_owned(),
        }
    }

    fn section(&self) -> usize {
        match self.page_tabs.active().or(self.page_tabs.cursor()) {
            Some(ItemKey::Index(index)) => index.min(SECTION_NAMES.len().saturating_sub(1)),
            _ => 0,
        }
    }

    fn unsaved(&self) -> Stakes {
        if self.title == self.saved {
            Stakes::Clean
        } else {
            Stakes::Unsaved { changes: 1 }
        }
    }

    fn work(&self) -> Stakes {
        if self.running {
            Stakes::Running {
                what: "the export".to_owned(),
            }
        } else {
            Stakes::Clean
        }
    }

    fn stakes(&self) -> Stakes {
        self.unsaved().max(self.work())
    }

    fn quit_keys(&self) -> QuitKeys {
        let letter = match self.options[2] {
            1 => Some('x'),
            2 => None,
            _ => Some('q'),
        };
        let mut keys = QuitKeys::DEFAULT
            .letter(letter)
            .esc_quits(self.options[3] == 1);
        keys.ctrl_c = self.options[4] == 1;
        keys.ctrl_q = self.options[5] == 1;
        keys
    }

    fn notes(&self) -> String {
        let contexts = contexts();
        let allowed = |context: KeyContext| termrock::shell::quit::letters_allowed(context);
        format!(
            "letters_allowed: clean {}, text focus {}, dirty {}. Quit goes through the guard; Copy copies a selection. Leave asks for Leave::To(Overview), Quit asks for Leave::Quit, and answers resolve to Stay, Save(to), or Discard(to). The shell quit request also asks while edits or an export are active.",
            allowed(contexts[0].1),
            allowed(contexts[1].1),
            allowed(contexts[2].1),
        )
    }

    fn playground_layout(&self, width: u16) -> PlaygroundLayout {
        let wide = width >= WIDE_LAYOUT;
        let document_height = 13;
        let key_width = if wide {
            width.saturating_sub(width / 2 + 1)
        } else {
            width
        };
        let note_width = key_width.saturating_sub(4).max(1);
        let note_rows = wrap(&self.notes(), note_width).len();
        let keys_height = QUIT_KEYS_NOTES_START_ROW
            .saturating_add(note_rows)
            .saturating_add(CARD_CHROME_ROWS)
            .max(12);
        let content_height = if wide {
            document_height.max(keys_height)
        } else {
            document_height
                .saturating_add(1)
                .saturating_add(keys_height)
        };
        PlaygroundLayout {
            wide,
            document_height,
            keys_height,
            content_height,
        }
    }

    fn states_layout(&self, width: u16) -> StatesLayout {
        let columns = if width >= 72 {
            4
        } else if width >= 38 {
            2
        } else {
            1
        };
        let header_rows = if columns == 2 { 2 } else { 1 };
        let sample_rows = (4usize.saturating_add(columns - 1)) / columns;
        let note_rows = wrap(
            "Hover or hold a sample to see its native pointer state. The Focused row starts on Subtle; Disabled controls cannot receive focus.",
            width.saturating_sub(2).max(1),
        )
        .len();
        StatesLayout {
            columns,
            header_rows,
            content_height: header_rows
                .saturating_add(STATE_NAMES.len().saturating_mul(sample_rows))
                .saturating_add(2)
                .saturating_add(note_rows),
        }
    }

    fn refresh_guard(&mut self) {
        self.guard = LeaveGuard::new(GUARD_DIALOG).save(self.options[1] == 1);
    }

    fn set_stakes_option(&mut self) {
        match self.options[0] {
            1 => {
                self.title = "Q3 settlement report (draft)".to_owned();
                self.saved = SAVED_TITLE.to_owned();
                self.running = false;
            }
            2 => {
                self.title = SAVED_TITLE.to_owned();
                self.saved = SAVED_TITLE.to_owned();
                self.running = true;
            }
            _ => {
                self.title = SAVED_TITLE.to_owned();
                self.saved = SAVED_TITLE.to_owned();
                self.running = false;
            }
        }
        self.title_state = TextInputState::default();
    }

    fn request(&mut self, to: Leave<&'static str>, cx: &mut Cx<'_>) -> Option<PageStatus> {
        let stakes = self.stakes();
        match self.guard.request(to, &stakes) {
            Ok(to) => {
                self.last_request = format!("Ok({}): go now", leave_label(&to));
                Some(PageStatus(format!("Would {} now", go_label(&to))))
            }
            Err(dialog) => {
                self.last_request = format!("Err: {}", dialog.title);
                let view = Self::leave_dialog(&dialog);
                cx.open_layer(GUARD_DIALOG, view.layer(cx));
                self.guard_dialog = Some(dialog);
                Some(PageStatus("Confirmation required".to_owned()))
            }
        }
    }

    fn leave_dialog(dialog: &LeaveDialog) -> Dialog<'_> {
        let actions: &'static [Action<'static>] = match dialog.kind {
            LeaveDialogKind::SaveDiscardCancel => &SAVE_ACTIONS,
            LeaveDialogKind::DiscardOnly => &DISCARD_ACTIONS,
            LeaveDialogKind::StopStay => &RUNNING_ACTIONS,
        };
        let view = Dialog::new(GUARD_DIALOG)
            .title(dialog.title)
            .description(&dialog.body)
            .actions(actions)
            .cancel(ActionKey::CANCEL)
            .width(62);
        if dialog.kind == LeaveDialogKind::SaveDiscardCancel {
            view.primary(ActionKey::SAVE)
        } else {
            view
        }
    }

    fn resolve_dialog(&mut self, action: DialogAction, cx: &mut Cx<'_>) -> PageStatus {
        let dialog_kind = self
            .guard_dialog
            .as_ref()
            .map(|dialog| dialog.kind)
            .unwrap_or(LeaveDialogKind::DiscardOnly);
        let answer = match action {
            DialogAction::Action(key) if key == ActionKey::DISCARD => DialogAnswer::Second,
            DialogAction::Action(key) if key == ActionKey::SAVE => DialogAnswer::Third,
            DialogAction::Action(key) if key == ActionKey::CONFIRM => DialogAnswer::Second,
            DialogAction::Action(_) | DialogAction::Dismissed(_) => DialogAnswer::First,
        };
        if cx.is_open(GUARD_DIALOG) {
            cx.close_layer(GUARD_DIALOG, None);
        }
        self.guard_dialog = None;
        let resolution = self.guard.resolve(answer);
        let (label, status) = match resolution {
            Resolved::Stay => ("Resolved::Stay".to_owned(), "Stayed".to_owned()),
            Resolved::Save(to) => {
                self.saved.clone_from(&self.title);
                (
                    format!("Resolved::Save({})", leave_label(&to)),
                    format!("Saved; would {} now", go_label(&to)),
                )
            }
            Resolved::Discard(to) if dialog_kind == LeaveDialogKind::StopStay => {
                self.running = false;
                self.title = self.saved.clone();
                self.title_state = TextInputState::default();
                (
                    format!("Resolved::Discard({})", leave_label(&to)),
                    format!("Stopped; would {} now", go_label(&to)),
                )
            }
            Resolved::Discard(to) => {
                self.title = self.saved.clone();
                self.title_state = TextInputState::default();
                self.running = false;
                (
                    format!("Resolved::Discard({})", leave_label(&to)),
                    format!("Discarded; would {} now", go_label(&to)),
                )
            }
        };
        self.last_answer = label;
        PageStatus(status)
    }

    fn update_scroll(
        cx: &mut Cx<'_>,
        id: Id,
        state: &mut ScrollState,
        content_len: usize,
    ) -> Response<()> {
        let mut response = ScrollRegion::new(id)
            .scrollbar_gutter(termrock::ScrollbarGutter::Stable)
            .update(cx, state, content_len);
        response |= update_scroll_keys(cx, id, state);
        response
    }

    fn update_playground_controls(
        &mut self,
        cx: &mut Cx<'_>,
    ) -> (Response<()>, Option<PageStatus>) {
        let mut response = Response::ignored();
        let mut status = None;
        if cx.area(TITLE).is_some() {
            let input = TextInput::new(TITLE).update(cx, &mut self.title_state, &mut self.title);
            response |= input.erase();
        }
        if cx.area(LEAVE).is_some() {
            let activation = leave_button().update(cx);
            if activation.activated() && !cx.is_open(GUARD_DIALOG) {
                status = self.request(Leave::To("Overview"), cx);
            }
            response |= activation.erase();
        }
        if cx.area(QUIT).is_some() {
            let activation = quit_button().update(cx);
            if activation.activated() && !cx.is_open(GUARD_DIALOG) {
                status = self.request(Leave::Quit, cx);
            }
            response |= activation.erase();
        }
        if cx.area(EXPORT).is_some() {
            let activation = export_button(self.running).update(cx);
            if activation.activated() {
                self.running = !self.running;
                status = Some(PageStatus(if self.running {
                    "Export running".to_owned()
                } else {
                    "Export stopped".to_owned()
                }));
            }
            response |= activation.erase();
        }
        (response, status)
    }

    fn update_options(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        let mut response = Response::ignored();
        for index in 0..OPTION_IDS.len() {
            let id = option_id(index);
            if (self.section() == 1 && cx.area(id).is_some())
                || self.option_states[index].is_open()
                || cx.is_open(id)
            {
                let previous = self.options[index];
                let result = Select::new(id).popup_rows(3).update_in_form(
                    cx,
                    &mut self.option_states[index],
                    &mut self.options[index],
                    OPTION_ITEMS[index],
                    false,
                );
                if self.options[index] != previous && index == 0 {
                    self.set_stakes_option();
                }
                if self.options[index] != previous && index == 1 {
                    self.refresh_guard();
                }
                response |= result.erase();
            }
        }
        response
    }

    fn update_state_buttons(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<PageStatus>) {
        let mut response = Response::ignored();
        let mut status = None;
        if self.section() != 2 {
            return (response, status);
        }
        for row in 0..STATE_NAMES.len() {
            for column in 0..STATE_ACTIONS.len() {
                let id = state_button_id(row, column);
                if cx.area(id).is_none() {
                    continue;
                }
                let activation = state_button(row, column).update(cx);
                if activation.activated() {
                    self.last_sample = format!(
                        "{} · {} · {}",
                        STATE_NAMES[row], VARIANT_NAMES[column], STATE_ACTIONS[column]
                    );
                    status = Some(PageStatus(format!("{} sample activated", STATE_NAMES[row])));
                }
                response |= activation.erase();
            }
        }
        (response, status)
    }

    fn document_panel(
        &self,
        ui: &mut Ui<'_>,
        x: u16,
        width: u16,
        height: usize,
        page_window: VerticalWindow,
        source_start: usize,
    ) {
        let Some(panel_window) = nested_window(page_window, source_start, height, x, width) else {
            return;
        };
        let panel_y = u16::try_from(source_start).unwrap_or(u16::MAX);
        let logical_area = Rect::new(x, panel_y, width, u16::try_from(height).unwrap_or(u16::MAX));
        let meta = if self.running {
            "exporting"
        } else if self.title == self.saved {
            "saved"
        } else {
            "edited"
        };
        let _ = card_panel_window(
            ui,
            Id::root("showcase_ui::pages::quit_leave.document_panel"),
            "Document",
            Some(meta),
            logical_area,
            panel_window,
            |ui, body_window| {
                let Some(body_window) = body_window else {
                    return;
                };
                let visible = body_window.visible_body();
                if let Some(y) = window_row(body_window, 0) {
                    let label_style = heading_style(ui);
                    ui.paint_str(
                        Rect::new(visible.x, y, 7.min(visible.width), 1),
                        "Title",
                        label_style,
                    );
                    let x = visible.x.saturating_add(7);
                    let input_width = visible.width.saturating_sub(7);
                    TextInput::new(TITLE).value(&self.title).draw(
                        ui,
                        Rect::new(x, y, input_width, 1),
                        &self.title_state,
                    );
                }
                if let Some(y) = window_row(body_window, 2) {
                    let mut x = visible.x;
                    let row = Rect::new(visible.x, y, visible.width, 1);
                    for button in [leave_button(), quit_button(), export_button(self.running)] {
                        if x >= row.right() {
                            break;
                        }
                        let drawn =
                            button.draw(ui, Rect::new(x, y, row.right().saturating_sub(x), 1));
                        x = drawn.right().saturating_add(1);
                    }
                }
                let facts = [
                    ("Edits", stakes_label(&self.unsaved())),
                    ("Work", stakes_label(&self.work())),
                    ("Stakes::max", stakes_label(&self.stakes())),
                    (
                        "LeaveGuard",
                        if self.guard.save {
                            "save: Cancel Discard Save".to_owned()
                        } else {
                            "no save: Cancel Discard".to_owned()
                        },
                    ),
                    ("Last request", self.last_request.clone()),
                    ("Last answer", self.last_answer.clone()),
                ];
                for (index, (name, value)) in facts.iter().enumerate() {
                    let source_row = index + 4;
                    if let Some(y) = window_row(body_window, source_row) {
                        let label_style = text_style(ui);
                        let label_width = 12.min(visible.width);
                        ui.paint_str(Rect::new(visible.x, y, label_width, 1), name, label_style);
                        let value_x = visible.x.saturating_add(label_width);
                        ui.paint_str(
                            Rect::new(value_x, y, visible.right().saturating_sub(value_x), 1),
                            value,
                            label_style,
                        );
                    }
                }
            },
        );
    }

    fn quit_keys_panel(
        &self,
        ui: &mut Ui<'_>,
        x: u16,
        width: u16,
        height: usize,
        page_window: VerticalWindow,
        source_start: usize,
    ) {
        let Some(panel_window) = nested_window(page_window, source_start, height, x, width) else {
            return;
        };
        let panel_y = u16::try_from(source_start).unwrap_or(u16::MAX);
        let logical_area = Rect::new(x, panel_y, width, u16::try_from(height).unwrap_or(u16::MAX));
        let _ = card_panel_window(
            ui,
            Id::root("showcase_ui::pages::quit_leave.quit_keys_panel"),
            "QuitKeys::classify",
            Some("→ LeaveKey"),
            logical_area,
            panel_window,
            |ui, body_window| {
                let Some(body_window) = body_window else {
                    return;
                };
                let visible = body_window.visible_body();
                let keys = self.quit_keys();
                let letter_label = keys
                    .letter
                    .map_or_else(|| "letter".to_owned(), |c| c.to_string());
                let cases = [
                    (
                        letter_label,
                        plain_key(KeyCode::Char(keys.letter.unwrap_or('q'))),
                    ),
                    ("Ctrl+C".to_owned(), ctrl_key('c')),
                    ("Ctrl+Q".to_owned(), ctrl_key('q')),
                    ("Esc".to_owned(), plain_key(KeyCode::Esc)),
                ];
                let contexts = contexts();
                let label_width = 12.min(visible.width);
                let columns = 4usize;
                let column_width = visible.width.saturating_sub(label_width) / columns as u16;
                let key_style = key_hint_style(ui);
                let meta_style = text_style(ui);
                if let Some(y) = window_row(body_window, 0) {
                    for (column, (label, _)) in cases.iter().enumerate() {
                        let col_x = visible
                            .x
                            .saturating_add(label_width)
                            .saturating_add((column as u16).saturating_mul(column_width));
                        ui.paint_str(Rect::new(col_x, y, column_width, 1), label, key_style);
                    }
                }
                for (row, (name, context)) in contexts.iter().enumerate() {
                    let source_row = row + 1;
                    if let Some(y) = window_row(body_window, source_row) {
                        ui.paint_str(Rect::new(visible.x, y, label_width, 1), name, meta_style);
                        for (column, (_, key)) in cases.iter().enumerate() {
                            let value = match keys.classify(key, *context) {
                                Some(LeaveKey::Quit) => "Quit",
                                Some(LeaveKey::Copy) => "Copy",
                                None => "—",
                            };
                            let col_x = visible
                                .x
                                .saturating_add(label_width)
                                .saturating_add((column as u16).saturating_mul(column_width));
                            ui.paint_str(Rect::new(col_x, y, column_width, 1), value, meta_style);
                        }
                    }
                }
                let note = self.notes();
                let notes_width = visible.width.max(1);
                for (index, line) in wrap(&note, notes_width).iter().enumerate() {
                    if let Some(y) = window_row(body_window, index + QUIT_KEYS_NOTES_START_ROW) {
                        ui.paint_str(Rect::new(visible.x, y, visible.width, 1), line, meta_style);
                    }
                }
            },
        );
    }

    fn draw_playground_content(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let visible = window.visible_body();
        let layout = self.playground_layout(visible.width);
        let doc_width = if layout.wide {
            visible.width / 2
        } else {
            visible.width
        };
        if layout.wide {
            let keys_x = visible.x.saturating_add(doc_width).saturating_add(1);
            let keys_width = visible.width.saturating_sub(doc_width).saturating_sub(1);
            self.document_panel(ui, visible.x, doc_width, layout.content_height, window, 0);
            self.quit_keys_panel(ui, keys_x, keys_width, layout.content_height, window, 0);
        } else {
            self.document_panel(ui, visible.x, doc_width, layout.document_height, window, 0);
            let keys_start = layout.document_height.saturating_add(1);
            self.quit_keys_panel(
                ui,
                visible.x,
                visible.width,
                layout.keys_height,
                window,
                keys_start,
            );
        }
    }

    fn draw_playground(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let content_width = area.width.saturating_sub(1);
        let content_height = self.playground_layout(content_width).content_height;
        let view = ScrollRegion::new(PLAY_SCROLL)
            .scrollbar_gutter(termrock::ScrollbarGutter::Stable)
            .draw(ui, area, &self.play_scroll, content_height);
        ui.register_focus_only(PLAY_SCROLL, Focusability::Focusable);
        register_scroll_keys(ui, PLAY_SCROLL);
        if let Some(window) = scroll_window(view, content_height, &self.play_scroll) {
            ui.with_area(view, |ui| self.draw_playground_content(ui, window));
        }
    }

    fn draw_options(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let content_height = OPTIONS_CONTENT_ROWS;
        let view = ScrollRegion::new(OPTIONS_SCROLL)
            .scrollbar_gutter(termrock::ScrollbarGutter::Stable)
            .draw(ui, area, &self.options_scroll, content_height);
        ui.register_focus_only(OPTIONS_SCROLL, Focusability::Focusable);
        register_scroll_keys(ui, OPTIONS_SCROLL);
        let Some(window) = scroll_window(view, content_height, &self.options_scroll) else {
            return;
        };
        let Some(panel_window) = nested_window(window, 0, content_height, view.x, view.width)
        else {
            return;
        };
        let _ = card_panel_window(
            ui,
            Id::root("showcase_ui::pages::quit_leave.options_panel"),
            "Options",
            Some("leave guard and key classification"),
            Rect::new(view.x, 0, view.width, content_height as u16),
            panel_window,
            |ui, body_window| {
                let Some(body_window) = body_window else {
                    return;
                };
                let visible = body_window.visible_body();
                let label_style = heading_style(ui);
                for index in 0..OPTION_IDS.len() {
                    let Some(y) = window_row(body_window, index.saturating_mul(2)) else {
                        continue;
                    };
                    let label_width = LABEL_WIDTH.min(visible.width);
                    ui.paint_str(
                        Rect::new(visible.x, y, label_width, 1),
                        OPTION_LABELS[index],
                        label_style,
                    );
                    let select_x = visible.x.saturating_add(label_width);
                    let select_width = visible.right().saturating_sub(select_x);
                    Select::new(option_id(index)).popup_rows(3).draw(
                        ui,
                        Rect::new(select_x, y, select_width, 1),
                        &self.option_states[index],
                        OPTION_ITEMS[index],
                    );
                    if let Some(note_y) =
                        window_row(body_window, index.saturating_mul(2).saturating_add(1))
                    {
                        let value = option_name(index, self.options[index]);
                        let value_style = text_style(ui);
                        ui.paint_str(
                            Rect::new(select_x, note_y, select_width, 1),
                            value,
                            value_style,
                        );
                    }
                }
            },
        );
    }

    fn draw_state_content(&self, ui: &mut Ui<'_>, window: VerticalWindow) {
        let visible = window.visible_body();
        let layout = self.states_layout(visible.width);
        let columns = layout.columns;
        let label_width = if columns == 4 {
            10
        } else if columns == 2 {
            9
        } else {
            14
        };
        let column_width = visible
            .width
            .saturating_sub(label_width)
            .checked_div(u16::try_from(columns).unwrap_or(1))
            .unwrap_or(0);
        let heading = heading_style(ui);
        let meta = text_style(ui);
        let note = "Hover or hold a sample to see its native pointer state. The Focused row starts on Subtle; Disabled controls cannot receive focus.";
        if let Some(y) = window_row(window, 0) {
            if columns == 4 {
                for column in 0..4 {
                    let x = visible
                        .x
                        .saturating_add(label_width)
                        .saturating_add((column as u16).saturating_mul(column_width));
                    ui.paint_str(
                        Rect::new(x, y, column_width, 1),
                        VARIANT_NAMES[column],
                        heading,
                    );
                }
            } else if columns == 2 {
                for column in 0..2 {
                    let x = visible
                        .x
                        .saturating_add(label_width)
                        .saturating_add((column as u16).saturating_mul(column_width));
                    ui.paint_str(
                        Rect::new(x, y, column_width, 1),
                        VARIANT_NAMES[column],
                        heading,
                    );
                }
                if let Some(y) = window_row(window, 1) {
                    for column in 0..2 {
                        let x = visible
                            .x
                            .saturating_add(label_width)
                            .saturating_add((column as u16).saturating_mul(column_width));
                        ui.paint_str(
                            Rect::new(x, y, column_width, 1),
                            VARIANT_NAMES[column + 2],
                            heading,
                        );
                    }
                }
            } else {
                ui.paint_str(
                    Rect::new(visible.x, y, visible.width, 1),
                    "State · variant · action",
                    heading,
                );
            }
        }
        let mut source_row = layout.header_rows;
        for row in 0..STATE_NAMES.len() {
            let row_start = source_row;
            let subrows = (STATE_ACTIONS.len().saturating_add(columns - 1)) / columns;
            for column in 0..STATE_ACTIONS.len() {
                let subrow = column / columns;
                let within = column % columns;
                let Some(y) = window_row(window, row_start.saturating_add(subrow)) else {
                    continue;
                };
                let label = if columns == 1 {
                    format!("{} · {}", STATE_NAMES[row], VARIANT_NAMES[column])
                } else if subrow == 0 {
                    STATE_NAMES[row].to_owned()
                } else {
                    String::new()
                };
                if !label.is_empty() {
                    ui.paint_str(
                        Rect::new(visible.x, y, label_width.min(visible.width), 1),
                        &label,
                        meta,
                    );
                }
                let x = visible
                    .x
                    .saturating_add(label_width)
                    .saturating_add((within as u16).saturating_mul(column_width));
                let right = visible
                    .x
                    .saturating_add(label_width)
                    .saturating_add(((within + 1) as u16).saturating_mul(column_width));
                state_button(row, column).draw(ui, Rect::new(x, y, right.saturating_sub(x), 1));
            }
            source_row = source_row.saturating_add(subrows);
        }
        if let Some(y) = window_row(window, source_row.saturating_add(1)) {
            let last = format!("Last sample: {}", self.last_sample);
            ui.paint_str(Rect::new(visible.x, y, visible.width, 1), &last, meta);
        }
        let note_start = source_row.saturating_add(2);
        for (index, line) in wrap(note, visible.width.max(1))
            .iter()
            .enumerate()
        {
            if let Some(y) = window_row(window, note_start.saturating_add(index)) {
                ui.paint_str(Rect::new(visible.x, y, visible.width, 1), line, meta);
            }
        }
    }

    fn draw_states(&self, ui: &mut Ui<'_>, area: Rect) {
        if area.is_empty() {
            return;
        }
        let content_width = area.width.saturating_sub(1);
        let content_height = self.states_layout(content_width).content_height;
        let view = ScrollRegion::new(STATES_SCROLL)
            .scrollbar_gutter(termrock::ScrollbarGutter::Stable)
            .draw(ui, area, &self.states_scroll, content_height);
        ui.register_focus_only(STATES_SCROLL, Focusability::Focusable);
        register_scroll_keys(ui, STATES_SCROLL);
        if let Some(window) = scroll_window(view, content_height, &self.states_scroll) {
            ui.with_area(view, |ui| self.draw_state_content(ui, window));
        }
    }

    fn active_option_control(&self, ui: &Ui<'_>) -> bool {
        OPTION_IDS
            .iter()
            .any(|id| ui.state(*id).contains(StateFlags::FOCUSED))
    }

    fn active_sample_button(&self, ui: &Ui<'_>) -> bool {
        (0..STATE_NAMES.len()).any(|row| {
            (0..STATE_ACTIONS.len()).any(|column| {
                ui.state(state_button_id(row, column))
                    .contains(StateFlags::FOCUSED)
            })
        })
    }
}

impl Default for QuitLeavePage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for QuitLeavePage {
    fn title(&self) -> &'static str {
        "Quit & leave"
    }

    fn quit_confirmation(&self) -> Option<&'static str> {
        match self.stakes() {
            Stakes::Clean => None,
            Stakes::Unsaved { .. } => Some("The document has unsaved changes."),
            Stakes::Running { .. } => Some("The export is still running."),
        }
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let before = self.section();
        let mut response = Tabs::new(PAGE_TABS)
            .update(cx, &mut self.page_tabs, SECTION_NAMES)
            .erase();
        let section = self.section();
        if before != section {
            match section {
                0 => self.play_scroll.scroll_to(0),
                1 => self.options_scroll.scroll_to(0),
                2 => self.states_scroll.scroll_to(0),
                _ => {}
            }
        }
        let width = cx
            .area(match section {
                1 => OPTIONS_SCROLL,
                2 => STATES_SCROLL,
                _ => PLAY_SCROLL,
            })
            .map_or(0, |area| area.width.saturating_sub(1));
        let mut status = None;
        match section {
            0 => {
                let layout = self.playground_layout(width);
                if cx.area(PLAY_SCROLL).is_some() {
                    response |= Self::update_scroll(
                        cx,
                        PLAY_SCROLL,
                        &mut self.play_scroll,
                        layout.content_height,
                    );
                }
                let (controls, control_status) = self.update_playground_controls(cx);
                response |= controls;
                status = control_status;
            }
            1 => {
                let content_height = OPTIONS_CONTENT_ROWS;
                if cx.area(OPTIONS_SCROLL).is_some() {
                    response |= Self::update_scroll(
                        cx,
                        OPTIONS_SCROLL,
                        &mut self.options_scroll,
                        content_height,
                    );
                }
                response |= self.update_options(cx);
            }
            2 => {
                let content_height = self.states_layout(width).content_height;
                if cx.area(STATES_SCROLL).is_some() {
                    response |= Self::update_scroll(
                        cx,
                        STATES_SCROLL,
                        &mut self.states_scroll,
                        content_height,
                    );
                }
                let (buttons, button_status) = self.update_state_buttons(cx);
                response |= buttons;
                status = button_status;
            }
            _ => {}
        }
        if let Some(dialog) = self.guard_dialog.as_ref() {
            let update = Self::leave_dialog(dialog).update(cx, &mut self.guard_state);
            if let Some(action) = update.action_ref().copied() {
                status = Some(self.resolve_dialog(action, cx));
            }
            response |= update.erase();
        }
        PageUpdate { response, status }
    }

    fn option_value(&self, key: &str) -> Option<usize> {
        page_option_index(key).map(|index| self.options[index])
    }

    fn set_option(&mut self, key: &str, value: usize) {
        let Some(index) = page_option_index(key) else {
            return;
        };
        self.options[index] = value.min(OPTION_ITEMS[index].len().saturating_sub(1));
        if index == 0 {
            self.set_stakes_option();
        } else if index == 1 {
            self.refresh_guard();
        }
    }

    fn blocks_shell_commands(&self, cx: &Cx<'_>) -> bool {
        cx.is_open(GUARD_DIALOG)
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.title_state.is_editing()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let tabs_area = section_tabs_area(area);
        let tabs = Tabs::new(PAGE_TABS);
        let previous =
            ui.layout(PAGE_TABS)
                .map(|facts| termrock::navigation::tabs::TabWindowReceipt {
                    viewport_len: facts.viewport_len,
                    content_len: facts.content_len,
                });
        let tabs_layout =
            tabs.layout_default(tabs_area, area, &self.page_tabs, SECTION_NAMES, previous);
        let section = if tabs_layout.visible_area.is_empty() {
            0
        } else {
            self.section()
        };
        frame_with_body_rows(
            ui,
            area,
            self.title(),
            "QuitKeys, LeaveGuard, and native confirmation states",
            BODY_START_ROW,
            |ui, body| {
                if !tabs_layout.visible_area.is_empty() {
                    let _ = tabs.draw_layout(ui, &tabs_layout);
                }
                match section {
                    1 => self.draw_options(ui, body),
                    2 => self.draw_states(ui, body),
                    _ => self.draw_playground(ui, body),
                }
            },
        );
        if let Some(dialog) = self.guard_dialog.as_ref() {
            let view = Self::leave_dialog(dialog);
            ui.layer(GUARD_DIALOG, |ui, area| {
                view.draw(ui, area, &self.guard_state, |_, _| {});
            });
        }
    }

    fn hints(&self, ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.guard_dialog.is_some() {
            return DIALOG_HINTS;
        }
        if ui.state(PAGE_TABS).contains(StateFlags::FOCUSED) {
            return TAB_HINTS;
        }
        if ui.state(TITLE).contains(StateFlags::FOCUSED) {
            return TEXT_HINTS;
        }
        if self.active_option_control(ui) {
            return OPTION_HINTS;
        }
        if ui.state(PLAY_SCROLL).contains(StateFlags::FOCUSED)
            || ui.state(OPTIONS_SCROLL).contains(StateFlags::FOCUSED)
            || ui.state(STATES_SCROLL).contains(StateFlags::FOCUSED)
        {
            return SCROLL_HINTS;
        }
        if ui.state(LEAVE).contains(StateFlags::FOCUSED)
            || ui.state(QUIT).contains(StateFlags::FOCUSED)
            || ui.state(EXPORT).contains(StateFlags::FOCUSED)
            || self.active_sample_button(ui)
        {
            return BUTTON_HINTS;
        }
        match self.section() {
            1 => OPTION_HINTS,
            2 => SCROLL_HINTS,
            _ => BUTTON_HINTS,
        }
    }
}

fn section_tabs_area(area: Rect) -> Rect {
    let y = area.y.saturating_add(2).min(area.bottom());
    Rect {
        y,
        height: area.bottom().saturating_sub(y).min(2),
        ..area
    }
}
