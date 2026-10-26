use termrock::shell::{ReportContent, TextReport};

pub(crate) fn draw(world: &World, ui: &mut Ui<'_>, area: Rect) {
    if area.is_empty() {
        return;
    }
    let title = format!("Disk usage · {}", world.cwd);
    let Some(disk) = &world.disk else {
        TextReport::new().min_details_height(0).draw(
            ui,
            area,
            &ReportContent::heading(&title).with_empty("No disk scan is open"),
        );
        return;
    };
    let subtitle = format!(
        "{}% full · {} entries",
        disk.used_percent(),
        disk.candidates.len()
    );
    let body = disk.candidates.iter().map(|candidate| {
        format!(
            "{} · {} · {}",
            candidate.path,
            candidate.family.label(),
            candidate.freshness.label()
        )
    });
    TextReport::new().min_details_height(0).draw_rows(
        ui,
        area,
        &ReportContent::heading(&title).with_subheading(&subtitle),
        body,
    );
}

use crate::domain::{
    disk::{Candidate, DiskState, Freshness},
    host::Host,
};
use crate::sim::{
    plans::{ReviewedPlan, plan_for_disk_selection},
    world::World,
};
use std::fmt;
use termrock::{
    Action, ActionKey, Cx, Dialog, DialogAction, DialogState, EmptyState, FgStep, FrameRead, Id,
    Intent, ItemKey, List, ListState, Rect, Response, Role, SelectMode, StylePatch, TextSection,
    Ui,
};

pub(crate) const SELECTION_DIALOG: Id = Id::root("holla.disk.selection");
pub(crate) const SELECTION_LIST: Id = Id::root("holla.disk.selection.list");
const DIALOG_TITLE: &str = "Select disk candidates";
const DIALOG_DESCRIPTION: &str = "Choose exact cleanup targets · activity unknown may be selected manually · active today is locked";
const ACTIONS_EMPTY: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "Review plan").enabled(false),
];
const ACTIONS_READY: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "Review plan"),
];

#[derive(Clone, Copy)]
struct OwnerPoll {
    safe: bool,
    has_intents: bool,
}

/// Inspect an owner without claiming layer receipts. A later native Dialog/List
/// update gets the same unclaimed lifecycle intents when the whole owner set is safe.
fn inspect_retirement_owner(cx: &Cx<'_>, owner: Id, allow_layer: bool) -> OwnerPoll {
    let mut safe = true;
    let mut has_intents = false;
    for intent in cx.intents(owner) {
        has_intents = true;
        match intent {
            Intent::FocusIn { .. } | Intent::FocusOut { .. } => {}
            Intent::Layer(_) if allow_layer => {}
            _ => safe = false,
        }
    }
    OwnerPoll { safe, has_intents }
}

#[derive(Clone)]
struct CandidateRow {
    candidate: Candidate,
}

impl fmt::Display for CandidateRow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} · {} · {} · {}",
            self.candidate.freshness.label(),
            self.candidate.family.label(),
            crate::domain::human_bytes(self.candidate.size_bytes),
            self.candidate.path
        )
    }
}

fn candidate_key(row: &CandidateRow) -> ItemKey {
    ItemKey::text(&row.candidate.path)
}

fn candidate_disabled(row: &CandidateRow) -> bool {
    row.candidate.freshness == Freshness::ActiveToday
}

fn candidate_list<'a>() -> List<'a, CandidateRow, impl Fn(&CandidateRow) -> ItemKey> {
    List::new(SELECTION_LIST)
        .key(candidate_key)
        .select_mode(SelectMode::Multi)
        .disabled_item(&candidate_disabled)
}

#[derive(Clone)]
struct OpeningBinding {
    host: Host,
    cwd: String,
    effect_revision: u64,
    disk: DiskState,
}

impl OpeningBinding {
    fn matches(&self, world: &World) -> bool {
        self.host == world.host
            && self.cwd == world.cwd
            && self.effect_revision == world.effect_revision
            && world.disk.as_ref() == Some(&self.disk)
    }
}

pub(crate) enum SelectionEvent {
    Cancelled,
    Stale,
    Invalid,
    Review(ReviewedPlan),
}

pub(crate) struct SelectionChooser {
    binding: OpeningBinding,
    rows: Vec<CandidateRow>,
    selection: ListState,
    dialog: DialogState,
    summary: String,
}

impl SelectionChooser {
    fn new(world: &World) -> Option<Self> {
        let disk = world.disk.as_ref()?;
        disk.validate(&world.cwd).ok()?;
        let mut chooser = Self {
            binding: OpeningBinding {
                host: world.host.clone(),
                cwd: world.cwd.clone(),
                effect_revision: world.effect_revision,
                disk: disk.clone(),
            },
            rows: disk
                .candidates
                .iter()
                .cloned()
                .map(|candidate| CandidateRow { candidate })
                .collect(),
            selection: ListState::default(),
            dialog: DialogState::default(),
            summary: String::new(),
        };
        chooser.refresh_summary();
        Some(chooser)
    }

    pub(crate) fn open(world: &World, cx: &mut Cx<'_>) -> Option<Self> {
        let chooser = Self::new(world)?;
        let props = chooser.props();
        cx.open_layer(SELECTION_DIALOG, props.layer(cx));
        cx.focus(SELECTION_LIST);
        Some(chooser)
    }

    fn selected_total(&self) -> Option<(usize, u64)> {
        let mut count = 0usize;
        let mut bytes = 0u64;
        for row in &self.rows {
            if self.selection.checked().contains(candidate_key(row)) {
                count = count.saturating_add(1);
                bytes = bytes.checked_add(row.candidate.size_bytes)?;
            }
        }
        Some((count, bytes))
    }

    fn refresh_summary(&mut self) {
        self.summary = match self.selected_total() {
            Some((count, bytes)) => format!(
                "{count} selected · {} · nothing changes until Plan review",
                crate::domain::human_bytes(bytes)
            ),
            None => "Selection total unavailable · reopen the Disk view".to_owned(),
        };
    }

    fn can_review(&self) -> bool {
        self.selected_total().is_some_and(|(count, _)| count > 0)
    }

    fn selected_paths(&self) -> Vec<String> {
        self.rows
            .iter()
            .filter(|row| self.selection.checked().contains(candidate_key(row)))
            .map(|row| row.candidate.path.clone())
            .collect()
    }

    fn props(&self) -> Dialog<'static> {
        let actions = if self.can_review() {
            &ACTIONS_READY
        } else {
            &ACTIONS_EMPTY
        };
        let rows = self.rows.len().saturating_add(1).min(usize::from(u16::MAX)) as u16;
        Dialog::new(SELECTION_DIALOG)
            .title(DIALOG_TITLE)
            .description(DIALOG_DESCRIPTION)
            .actions(actions)
            .cancel(ActionKey::CANCEL)
            .primary(ActionKey::CONFIRM)
            .body_rows(rows.max(2))
            .width(78)
    }

    fn update_widgets(&mut self, cx: &mut Cx<'_>) -> (Response<()>, Option<DialogAction>) {
        let mut response = candidate_list()
            .update(cx, &mut self.selection, &self.rows)
            .erase();
        self.refresh_summary();
        let mut dialog_response = self.props().update(cx, &mut self.dialog);
        let action = dialog_response.take_action();
        response |= dialog_response.erase();
        (response, action)
    }

    pub(crate) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        world: &World,
    ) -> (Response<()>, Option<SelectionEvent>, bool) {
        let (response, action) = self.update_widgets(cx);
        if !self.binding.matches(world) {
            self.selection = ListState::default();
            self.refresh_summary();
            let close_key = match action {
                Some(DialogAction::Action(key)) => key,
                Some(DialogAction::Dismissed(_)) | None => ActionKey::CANCEL,
            };
            cx.close_layer(SELECTION_DIALOG, Some(close_key));
            return (response, Some(SelectionEvent::Stale), true);
        }

        let event = match action {
            Some(DialogAction::Action(ActionKey::CONFIRM)) => {
                let paths = self.selected_paths();
                let review = (!paths.is_empty())
                    .then(|| plan_for_disk_selection(world, &paths))
                    .flatten();
                cx.close_layer(SELECTION_DIALOG, Some(ActionKey::CONFIRM));
                Some(review.map_or(SelectionEvent::Invalid, SelectionEvent::Review))
            }
            Some(DialogAction::Action(ActionKey::CANCEL) | DialogAction::Dismissed(_)) => {
                cx.close_layer(SELECTION_DIALOG, Some(ActionKey::CANCEL));
                Some(SelectionEvent::Cancelled)
            }
            Some(DialogAction::Action(_)) | None => None,
        };
        let closed = event.is_some() || !cx.is_open(SELECTION_DIALOG);
        (response, event, closed)
    }

    fn selection_action_owners(&self) -> [Id; 2] {
        let props = self.props();
        [props.action_id(0), props.action_id(1)]
    }

    /// Service only safe lifecycle buckets while hidden or in a too-small frame.
    /// Dialog.update remains the owner of its Layer receipts and dismissal state.
    fn service_lifecycle(&mut self, cx: &mut Cx<'_>) -> (Response<()>, bool) {
        let list = inspect_retirement_owner(cx, SELECTION_LIST, false);
        let actions = self.selection_action_owners();
        let root = inspect_retirement_owner(cx, SELECTION_DIALOG, true);
        let action_polls = actions.map(|owner| inspect_retirement_owner(cx, owner, false));
        let dialog_safe = root.safe && action_polls.iter().all(|poll| poll.safe);

        let mut response = Response::ignored();
        if list.safe {
            response |= candidate_list()
                .update(cx, &mut self.selection, &self.rows)
                .erase();
            self.refresh_summary();
        } else {
            cx.diagnostic(termrock::Diagnostic::UndeliveredIntent {
                owner: SELECTION_LIST,
            });
        }

        if dialog_safe {
            let mut dialog_response = self.props().update(cx, &mut self.dialog);
            let _ = dialog_response.take_action();
            response |= dialog_response.erase();
        } else {
            // The broad Dialog update reads all of these owners. If one bucket
            // contains input, leave the entire Dialog state untouched and report
            // both unsupported input and any safe lifecycle buckets it cannot service.
            if !root.safe || root.has_intents {
                cx.diagnostic(termrock::Diagnostic::UndeliveredIntent {
                    owner: SELECTION_DIALOG,
                });
            }
            for (owner, poll) in actions.into_iter().zip(action_polls) {
                if !poll.safe || poll.has_intents {
                    cx.diagnostic(termrock::Diagnostic::UndeliveredIntent { owner });
                }
            }
        }

        (response, list.safe && dialog_safe)
    }

    /// TooSmall keeps the active chooser; safe focus/layer lifecycle still advances.
    pub(crate) fn poll(&mut self, cx: &mut Cx<'_>) -> Response<()> {
        self.service_lifecycle(cx).0
    }

    /// A retired chooser may be released only after its actual owners and layer
    /// disappear from the published frame, focus ring, and pointer capture.
    pub(crate) fn poll_retired(&mut self, cx: &mut Cx<'_>) -> (Response<()>, bool) {
        let (response, owners_safe) = self.service_lifecycle(cx);
        let actions = self.selection_action_owners();
        let owners = [SELECTION_LIST, SELECTION_DIALOG, actions[0], actions[1]];
        let focused = owners
            .iter()
            .any(|owner| cx.state(*owner).contains(termrock::StateFlags::FOCUSED));
        let captured = cx
            .capture_owner()
            .is_some_and(|captured| owners.contains(&captured));
        let no_area = owners.iter().all(|owner| cx.area(*owner).is_none());
        let layer_closed = !cx.is_open(SELECTION_DIALOG);
        let ready = owners_safe && layer_closed && no_area && !focused && !captured;
        if owners_safe && layer_closed && !ready {
            cx.request_repaint();
        }
        (response, ready)
    }

    #[cfg(test)]
    pub(crate) fn selected_paths_for_test(&self) -> Vec<String> {
        self.selected_paths()
    }

    pub(crate) fn draw(&self, ui: &mut Ui<'_>) {
        ui.layer(SELECTION_DIALOG, |ui, area| {
            self.props().draw(ui, area, &self.dialog, |ui, body| {
                let summary_area = Rect::new(body.x, body.y, body.width, 1);
                let summary_style =
                    ui.paint_patch(&StylePatch::new().set_fg(Role::Fg(FgStep::Secondary)));
                TextSection::new(&self.summary)
                    .inset(0)
                    .title_style(summary_style)
                    .render(ui, summary_area);
                let list_area = Rect::new(
                    body.x,
                    body.y.saturating_add(1),
                    body.width,
                    body.height.saturating_sub(1),
                );
                if self.rows.is_empty() {
                    candidate_list()
                        .empty(EmptyState::Empty {
                            title: "No disk candidates",
                            hint: Some("No cleanup choices are available"),
                        })
                        .draw(ui, list_area, &self.selection, &self.rows);
                } else {
                    candidate_list().draw(ui, list_area, &self.selection, &self.rows);
                }
            });
        });
    }
}
