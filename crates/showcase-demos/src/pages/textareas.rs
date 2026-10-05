//! Multiline editing and viewport scrolling.

use termrock::{
    Cx, Family, FgStep, Field, FieldError, Id, Panel, PanelKind, Part, Rect, Role, StateFlags,
    StylePatch, TextAction, TextArea, TextAreaState, Track, Ui, Variant, id, layout, truncate,
    width,
};

use super::{Page, PageUpdate, frame};

const BODY: Id = id!("textareas.body");
const NOTES: Id = id!("textareas.notes");
const TRANSCRIPT: Id = id!("textareas.transcript");
const COMMIT: Id = id!("textareas.commit");
const PLAYGROUND: Id = id!("textareas.playground");
const STATES: Id = id!("textareas.states");

fn body_area() -> TextArea<'static> {
    TextArea::new(BODY, 8).placeholder("Write a checklist")
}

fn checklist() -> String {
    (1..=28)
        .map(|line| match line % 4 {
            0 => format!("{line:>2}. Run the integration suite and attach the report. Review 日本語 · 👩‍💻 · cafe\u{301} through this long Unicode line."),
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

fn columns(area: Rect, left_w: u16, gap: u16) -> (Rect, Rect) {
    if area.width < left_w.saturating_add(gap).saturating_add(20) {
        let h = area.height / 2;
        return (
            Rect::new(area.x, area.y, area.width, h),
            Rect::new(
                area.x,
                area.y.saturating_add(h),
                area.width,
                area.height.saturating_sub(h),
            ),
        );
    }
    (
        Rect::new(area.x, area.y, left_w, area.height),
        Rect::new(
            area.x.saturating_add(left_w).saturating_add(gap),
            area.y,
            area.width.saturating_sub(left_w).saturating_sub(gap),
            area.height,
        ),
    )
}

fn scroll_help(ui: &mut Ui<'_>, area: Rect, st: &TextAreaState) {
    let row = Rect {
        x: area.x.saturating_add(2),
        y: area.y.saturating_add(9),
        width: area.width.saturating_sub(2),
        height: 1,
    };
    if row.is_empty() {
        return;
    }
    let help_style = ui
        .style(
            Family::FIELD,
            Variant::DEFAULT,
            Part::HELP,
            StateFlags::empty(),
        )
        .style;
    let faint_style = ui
        .surface_style()
        .patch(ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Faint))));

    let pos = if st.is_editing() {
        format!("ln {}/{}", st.cursor_pos().y + 1, st.line_count())
    } else if st.scroll().overflows() {
        let r = st.scroll().visible_range();
        format!("{}–{} of {}", r.start + 1, r.end, st.scroll().content_len())
    } else {
        String::new()
    };
    let pos = truncate(&pos, area.width.saturating_sub(3));
    let pos_w = if pos.is_empty() {
        0
    } else {
        width(&pos).saturating_add(3)
    };
    let msg_w = area.width.saturating_sub(2 + pos_w);
    let help = truncate("Home/End follows long lines · Esc finishes", msg_w);
    let _ = ui.paint_str(
        Rect {
            x: area.x.saturating_add(2),
            y: area.y.saturating_add(9),
            width: msg_w,
            height: 1,
        },
        &help,
        help_style,
    );
    if !pos.is_empty() {
        let px = area.right().saturating_sub(width(&pos).saturating_add(1));
        let pos_rect = Rect {
            x: px,
            y: area.y.saturating_add(9),
            width: width(&pos),
            height: 1,
        };
        let _ = ui.paint_str(pos_rect, &pos, faint_style);
    }
}

/// A controlled multi-line document with enough rows to exercise wheel and
/// keyboard scrolling at both baseline sizes.
#[derive(Debug)]
pub struct TextAreasPage {
    value: String,
    state: TextAreaState,
    last: &'static str,
}

impl TextAreasPage {
    pub fn new() -> Self {
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
        let _ = playground_panel("Enter Edit · Esc Done · Tab Next");
        let _ = states_panel();
        let _ = notes_field();
        let _ = transcript_field();
        let _ = commit_field();
        edit.erase().into()
    }

    fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let meta = "Multi-line editing, Unicode cursor motion, horizontal and vertical scrolling";
        frame(ui, area, self.title(), meta, |ui, body| {
            let regions = layout::rows(body, &[Track::Fixed(13), Track::Fixed(1), Track::Flex(1)]);
            let playground = regions.first().copied().unwrap_or(body);
            playground_panel("Enter Edit · Esc Done · Tab Next").draw(
                ui,
                playground,
                |ui, inner| {
                    let (task, notes) = columns(inner, inner.width / 2 - 2, 4);
                    task_field(&self.value).draw(ui, task, &self.state);
                    scroll_help(ui, task, &self.state);
                    ui.reference(None, |ui| {
                        notes_field().draw(ui, notes, &TextAreaState::default());
                    });
                },
            );
            if let Some(states) = regions.get(2).copied() {
                states_panel().draw(ui, states, |ui, inner| {
                    Self::draw_states(ui, inner);
                });
            }
        });
    }

    fn hints(&self, _ui: &Ui<'_>) -> &'static [(&'static str, &'static str)] {
        if self.state.is_editing() {
            &[
                ("Enter", "Newline"),
                ("Esc", "Done"),
                ("Shift+↑↓", "Select"),
                ("Tab", "Next"),
            ]
        } else {
            &[("Enter", "Edit"), ("↑ ↓", "Scroll")]
        }
    }

    fn editing(&self, _ui: &Ui<'_>) -> bool {
        self.state.is_editing()
    }
}

impl TextAreasPage {
    fn draw_states(ui: &mut Ui<'_>, inner: Rect) {
        let (transcript, commit) = columns(inner, inner.width / 2 - 2, 4);
        let mut commit_state = TextAreaState::default();
        commit_state.set_error(Some(FieldError::new(
            "Use the imperative mood and explain why",
        )));
        ui.reference(None, |ui| {
            transcript_field().draw(ui, transcript, &TextAreaState::default());
            commit_field().draw(ui, commit, &commit_state);
        });
    }
}
