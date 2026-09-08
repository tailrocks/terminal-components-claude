//! Multiline editing and viewport scrolling.

use junie_tui::{
    Cx, Family, Field, FieldError, Id, Panel, PanelKind, Part, Rect, StateFlags, TextAction,
    TextArea, TextAreaState, Track, Ui, Variant, id, layout, truncate,
};

use super::{Page, PageUpdate, frame};

const BODY: Id = id!("textareas.body");
const NOTES: Id = id!("textareas.notes");
const TRANSCRIPT: Id = id!("textareas.transcript");
const COMMIT: Id = id!("textareas.commit");
const PLAYGROUND: Id = id!("textareas.playground");
const STATES: Id = id!("textareas.states");

fn body_area() -> TextArea<'static> {
    TextArea::new(BODY, 8)
}

fn checklist() -> String {
    (1..=28)
        .map(|line| match line % 4 {
            0 => format!("{line:>2}. Run the integration suite and attach the report."),
            1 => format!("{line:>2}. Read src/api/billing.rs before touching invoices."),
            2 => format!("{line:>2}. Keep the public API stable; add, never rename."),
            _ => format!("{line:>2}. Open a PR against main with a clear summary."),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn task_field(value: &str) -> Field<'_, TextArea<'_>> {
    Field::new("Task description", body_area().value(value)).optional_suffix(false)
}

fn notes_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Notes",
        TextArea::new(NOTES, 8).placeholder("Anything the agent should know…"),
    )
    .optional_suffix(false)
    .help("Optional")
}

fn transcript_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Read-only transcript",
        TextArea::new(TRANSCRIPT, 4)
            .value("Junie: Reading 14 files…\nJunie: Plan ready. 3 steps.")
            .disabled(true),
    )
    .optional_suffix(false)
}

fn commit_field() -> Field<'static, TextArea<'static>> {
    Field::new(
        "Commit message",
        TextArea::new(COMMIT, 4).value("fix stuff"),
    )
    .optional_suffix(false)
    .error(Some("Use the imperative mood and explain why"))
}

/// Both phase paths build the same two cards (§13).
fn playground_panel(meta: &'static str) -> Panel<'static> {
    Panel::new(PLAYGROUND)
        .kind(PanelKind::Card)
        .title("Playground")
        .meta(meta)
}

fn states_panel() -> Panel<'static> {
    Panel::new(STATES)
        .kind(PanelKind::Card)
        .title("Disabled and error")
}

fn legacy_field_gutter(ui: &mut Ui<'_>, area: Rect, flags: StateFlags) {
    if area.is_empty() {
        return;
    }
    let field = ui.style(Family::FIELD, Variant::DEFAULT, Part::FIELD, flags);
    let mut gutter = ui
        .style(Family::FIELD, Variant::DEFAULT, Part::GUTTER, flags)
        .style;
    gutter = gutter.with_bg_from(field.style);
    if !flags.contains(StateFlags::FOCUSED) {
        gutter = gutter.with_fg_from_bg(field.style);
    }
    for offset in 0..area.height {
        let _ = ui.paint_str(
            Rect {
                y: area.y.saturating_add(offset),
                height: 1,
                ..area
            },
            "▎",
            gutter,
        );
    }
}

fn legacy_scroll_help(ui: &mut Ui<'_>, area: Rect) {
    let row = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(9),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    if row.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::FIELD,
            Variant::DEFAULT,
            Part::HELP,
            StateFlags::empty(),
        )
        .style;
    let meta_x = row.right().saturating_sub(10);
    let help_width = meta_x.saturating_sub(row.x).saturating_sub(2);
    let help = truncate("Enter inserts a newline · Esc finishes", help_width);
    let blank = " ".repeat(usize::from(row.width));
    let _ = ui.paint_str(row, &blank, style);
    let _ = ui.paint_str(row, &help, style);
    let _ = ui.paint_str(
        Rect {
            x: meta_x,
            width: 9,
            ..row
        },
        "1–8 of 28",
        style,
    );
}

fn legacy_textarea_placeholder(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let inner = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(1),
        width: area.width.saturating_sub(3),
        height: 1,
    };
    if inner.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::TEXTAREA,
            Variant::DEFAULT,
            Part::PLACEHOLDER,
            StateFlags::empty(),
        )
        .style;
    let fitted = truncate(text, inner.width);
    let blank = " ".repeat(usize::from(inner.width));
    let _ = ui.paint_str(inner, &blank, style);
    let _ = ui.paint_str(inner, &fitted, style);
}

fn legacy_field_error(ui: &mut Ui<'_>, area: Rect, text: &str) {
    let row = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(5),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    if row.is_empty() {
        return;
    }
    let style = ui
        .style(
            Family::FIELD,
            Variant::DEFAULT,
            Part::HELP,
            StateFlags::ERROR,
        )
        .style;
    let fitted = truncate(text, row.width);
    let blank = " ".repeat(usize::from(row.width));
    let _ = ui.paint_str(row, &blank, style);
    let _ = ui.paint_str(row, &fitted, style);
}

/// A controlled multi-line document with enough rows to exercise wheel and
/// keyboard scrolling at both baseline sizes.
#[derive(Debug)]
pub(crate) struct TextAreasPage {
    value: String,
    state: TextAreaState,
    last: &'static str,
}

impl TextAreasPage {
    pub(crate) fn new() -> Self {
        Self {
            value: checklist(),
            state: TextAreaState::default(),
            last: "ready",
        }
    }
}

impl Default for TextAreasPage {
    fn default() -> Self {
        Self::new()
    }
}

impl Page for TextAreasPage {
    fn title(&self) -> &'static str {
        "Text areas"
    }

    fn update(&mut self, cx: &mut Cx<'_>) -> PageUpdate {
        let edit = body_area().update(cx, &mut self.state, &mut self.value);
        if let Some(action) = edit.action_ref() {
            self.last = match action {
                TextAction::Changed => "draft changed",
                TextAction::Committed => "document committed",
                TextAction::Cancelled => "draft cancelled",
                TextAction::MoveNext | TextAction::MovePrev => "focus moved",
            };
        }
        // Both phases build the same cards and reference fields (§13). The
        // narrow-width meta variant is a paint-budget quirk decided in draw;
        // update builds the canonical wide form through the same constructor.
        let _ = playground_panel("Enter Edit · Esc Done · Tab Next ");
        let _ = states_panel();
        let _ = notes_field();
        let _ = transcript_field();
        let _ = commit_field();
        edit.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let meta = "Multi-line editing, wrapping cursor motion, scroll position";
        frame(ui, area, self.title(), meta, |ui, body| {
            let regions =
                layout::rows(body, &[Track::Fixed(13), Track::Fixed(1), Track::Fixed(10)]);
            let playground = regions.first().copied().unwrap_or(body);
            let playground = if body.width < 70 {
                Rect {
                    width: playground.width.saturating_sub(1),
                    ..playground
                }
            } else {
                playground
            };
            playground_panel(if body.width < 70 {
                "Enter Edit · Esc Done · Tab Next"
            } else {
                "Enter Edit · Esc Done · Tab Next "
            })
            .draw(ui, playground, |ui, inner| {
                let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
                let task = {
                    let area = columns.first().copied().unwrap_or(inner);
                    Rect {
                        width: area.width.saturating_sub(1),
                        ..area
                    }
                };
                task_field(&self.value).draw(ui, task, &self.state);
                legacy_field_gutter(
                    ui,
                    Rect {
                        y: task.y.saturating_add(1),
                        height: 8,
                        ..task
                    },
                    StateFlags::empty(),
                );
                legacy_scroll_help(ui, task);
                let notes = {
                    let area = columns.get(1).copied().unwrap_or(inner);
                    Rect {
                        width: area.width.saturating_sub(u16::from(body.width >= 70)),
                        ..area
                    }
                };
                ui.reference(None, |ui| {
                    notes_field().draw(ui, notes, &TextAreaState::default());
                    legacy_textarea_placeholder(ui, notes, "Anything the agent should know…");
                    legacy_field_gutter(
                        ui,
                        Rect {
                            y: notes.y.saturating_add(1),
                            height: 8,
                            ..notes
                        },
                        StateFlags::empty(),
                    );
                });
            });
            if let Some(states) = regions.get(2).copied() {
                states_panel().draw(ui, states, |ui, inner| {
                    Self::draw_states(ui, inner);
                });
            }
        });
    }
}

impl TextAreasPage {
    fn draw_states(ui: &mut Ui<'_>, inner: Rect) {
        let columns = layout::columns(inner, &[Track::Flex(1), Track::Flex(1)], 3);
        let mut commit_state = TextAreaState::default();
        commit_state.set_error(Some(FieldError::new(
            "Use the imperative mood and explain why",
        )));
        ui.reference(None, |ui| {
            let transcript = {
                let area = columns.first().copied().unwrap_or(inner);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            transcript_field().draw(ui, transcript, &TextAreaState::default());
            legacy_field_gutter(
                ui,
                Rect {
                    y: transcript.y.saturating_add(1),
                    height: 4,
                    ..transcript
                },
                StateFlags::DISABLED,
            );
            let commit = {
                let area = columns.get(1).copied().unwrap_or(inner);
                Rect {
                    width: area.width.saturating_sub(1),
                    ..area
                }
            };
            commit_field().draw(ui, commit, &commit_state);
            legacy_field_error(ui, commit, "Use the imperative mood and explain why");
            legacy_field_gutter(
                ui,
                Rect {
                    y: commit.y.saturating_add(1),
                    height: 4,
                    ..commit
                },
                StateFlags::ERROR,
            );
        });
    }
}
