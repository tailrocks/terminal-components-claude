use core::fmt;
use std::cell::Cell;
use std::sync::Weak;

use tablepro_domain::model::{HistoryEntry, HistorySource};
use termrock::{
    ActionKey, Binding, Button, Chord, Cx, Diagnostic, Dialog, DialogState, Dismiss, FrameRead, Id,
    Intent, KeyCode, LayerEvent, LayerId, LayerOpening, Part, Response, TextViewport, Ui,
    ViewportAction, ViewportLine, ViewportState, wrap,
};

use crate::footer::Hint;
use crate::tabs::{Tab, TabKey};
use crate::workbench::Workbench;

const ID: Id = Id::root("tablepro.history-detail");
const TEXT: Id = ID.sub("text");
const ESC_CLOSE: ActionKey = ActionKey::custom("tablepro.history-detail.close");
const ESC_CLOSE_BINDINGS: &[Binding<()>] = &[Binding {
    action: ESC_CLOSE,
    chord: Some(Chord::key(KeyCode::Esc)),
    cmd: (),
    label: "Close",
    priority: 1,
    visible: false,
}];
const MIN_WIDTH: u16 = 72;
const MIN_HEIGHT: u16 = 20;

pub(super) struct HistoryDetailRequest {
    key: TabKey,
    owner: Weak<()>,
    generation: u64,
    entry: HistoryEntry,
}

impl fmt::Debug for HistoryDetailRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HistoryDetailRequest")
            .field("key", &self.key)
            .field("generation", &self.generation)
            .field("owner", &"[identity]")
            .field("entry", &"[redacted]")
            .finish()
    }
}

impl HistoryDetailRequest {
    pub(super) fn capture(workbench: &Workbench, key: TabKey, entry: HistoryEntry) -> Option<Self> {
        if workbench.active_key() != Some(key)
            || !matches!(workbench.tab(key), Some(Tab::History(_)))
        {
            return None;
        }
        Some(Self {
            key,
            owner: workbench.owner_token(),
            generation: workbench.generation(key)?,
            entry,
        })
    }

    fn is_current(&self, workbench: &Workbench, route_visible: bool) -> bool {
        route_visible
            && workbench.matches_owner(&self.owner)
            && workbench.active_key() == Some(self.key)
            && workbench.generation(self.key) == Some(self.generation)
            && matches!(workbench.tab(self.key), Some(Tab::History(_)))
    }

    fn can_restore_focus(&self, workbench: &Workbench, route_visible: bool) -> bool {
        route_visible
            && workbench.matches_owner(&self.owner)
            && workbench.active_key() == Some(self.key)
            && matches!(workbench.tab(self.key), Some(Tab::History(_)))
    }

    fn text_lines(&self) -> Vec<String> {
        project_detail(&self.entry)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DrawWitness {
    opening: LayerOpening,
    width: u16,
    height: u16,
}

#[derive(Default)]
pub(super) struct HistoryDetailController {
    request: Option<HistoryDetailRequest>,
    opening: Option<LayerOpening>,
    retiring: bool,
    closed_receipt_seen: bool,
    focus_restored: bool,
    interactive: bool,
    drawn: Cell<Option<DrawWitness>>,
    state: DialogState,
    viewport: ViewportState,
}

impl fmt::Debug for HistoryDetailController {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HistoryDetailController")
            .field("open", &self.is_open())
            .field("interactive", &self.interactive)
            .field("retiring", &self.retiring)
            .field("request", &self.request)
            .finish_non_exhaustive()
    }
}

impl HistoryDetailController {
    pub(super) fn is_open(&self) -> bool {
        self.request.is_some() && self.opening.is_some()
    }

    pub(super) fn is_interactive(&self) -> bool {
        self.is_open() && self.interactive && !self.retiring
    }

    pub(super) fn has_selection(&self) -> bool {
        self.is_interactive() && self.viewport.selection().is_some()
    }

    pub(super) fn footer_hints(&self) -> Vec<Hint> {
        let mut hints = vec![
            Hint::new("Ctrl+C", "Copy selection"),
            Hint::new("↑↓", "Move"),
            Hint::new("PgUp/PgDn", "Page"),
            Hint::new("Home/End", "Start / End"),
        ];
        if self.has_selection() {
            hints.push(Hint::new("Esc", "Clear selection"));
        } else {
            hints.push(Hint::new("Esc", "Close"));
        }
        hints
    }

    pub(super) fn open(&mut self, cx: &mut Cx<'_>, request: HistoryDetailRequest) -> bool {
        if self.is_open() || cx.is_open(ID) || cx.top_layer() != LayerId::PAGE {
            return false;
        }
        let viewport = cx.viewport();
        if viewport.width < MIN_WIDTH || viewport.height < MIN_HEIGHT {
            return false;
        }
        let Some(dialog) =
            sized_dialog(&request.entry, viewport.width, viewport.height, cx.design())
        else {
            return false;
        };
        let Some(opening) = cx.open_layer_with_token(
            ID,
            dialog
                .layer(cx)
                .initial_focus(TEXT)
                .restore_focus(false)
                .dismiss(Dismiss {
                    esc: false,
                    outside_click: true,
                    ..Dismiss::ESC
                }),
        ) else {
            return false;
        };
        self.state = DialogState::default();
        self.viewport = ViewportState::default();
        self.viewport.set_follow(false);
        self.drawn.set(None);
        self.focus_restored = false;
        self.retiring = false;
        self.closed_receipt_seen = false;
        self.interactive = true;
        self.opening = Some(opening);
        self.request = Some(request);
        true
    }

    pub(super) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        workbench: &Workbench,
        route_visible: bool,
    ) -> Response<()> {
        let (Some(opening), Some(_)) = (self.opening, self.request.as_ref()) else {
            if self.opening.is_some() || self.request.is_some() {
                self.clear_private();
            }
            return Response::ignored();
        };
        let screen = cx.viewport();
        let request_current = self
            .request
            .as_ref()
            .is_some_and(|r| r.is_current(workbench, route_visible));
        let can_restore_focus = screen.width >= MIN_WIDTH
            && screen.height >= MIN_HEIGHT
            && self
                .request
                .as_ref()
                .is_some_and(|r| r.can_restore_focus(workbench, route_visible));
        let receipts_closed = self.take_owned_receipts(cx, opening);
        let current = cx.layer_opening(ID);
        if current != Some(opening) {
            let changed = self.set_interactive(false);
            self.retiring = true;
            self.drawn.set(None);
            if self.closed_receipt_seen {
                if current.is_none() && can_restore_focus {
                    self.restore_focus(cx, workbench, route_visible, true);
                }
                self.clear_private();
                return Response::changed();
            }
            return if changed || receipts_closed {
                Response::changed()
            } else {
                Response::ignored()
            };
        }
        if self.closed_receipt_seen {
            self.retiring = true;
        }

        let lines = self
            .request
            .as_ref()
            .map(HistoryDetailRequest::text_lines)
            .unwrap_or_default();
        let viewport_lines: Vec<_> = lines
            .iter()
            .map(|line| ViewportLine::Plain(line.as_str()))
            .collect();
        let fits = screen.width >= MIN_WIDTH && screen.height >= MIN_HEIGHT;
        let dialog = fits
            .then(|| {
                self.request
                    .as_ref()
                    .and_then(|r| sized_dialog(&r.entry, screen.width, screen.height, cx.design()))
            })
            .flatten();
        let top = opening.layer_id() == cx.top_layer();
        let lifecycle_only = self.retiring || !request_current || dialog.is_none() || !top;
        if lifecycle_only {
            self.drawn.set(None);
            let mut response = if self.set_interactive(false) {
                Response::changed()
            } else {
                Response::ignored()
            };
            response |= self.service_lifecycle_only(cx, opening, &viewport_lines);
            if top && (!request_current || dialog.is_none() || self.retiring) {
                return response
                    | self.close_when_top(
                        cx,
                        workbench,
                        route_visible,
                        opening,
                        request_current && can_restore_focus && fits,
                    );
            }
            return response;
        }
        let Some(dialog) = dialog else {
            return Response::ignored();
        };
        let mut response = if self.set_interactive(true) {
            Response::changed()
        } else {
            Response::ignored()
        };

        if cx.layer_opening(ID) != Some(opening) || opening.layer_id() != cx.top_layer() {
            self.retiring = true;
            self.drawn.set(None);
            return response;
        }
        cx.resize_layer(ID, dialog.layer(cx).size);
        let witness = DrawWitness {
            opening,
            width: screen.width,
            height: screen.height,
        };
        let has_current_draw = self.drawn.get() == Some(witness)
            && cx.area(ID).is_some_and(|a| !a.is_empty())
            && cx.area(TEXT).is_some_and(|a| !a.is_empty());
        if !has_current_draw {
            response |= self.service_lifecycle_only(cx, opening, &viewport_lines);
            return response;
        }

        if self.viewport.selection().is_none()
            && cx.update_cause() == termrock::UpdateCause::Event
            && let Some(chord) = cx.effective_chord(TEXT, ESC_CLOSE, Some(Chord::key(KeyCode::Esc)))
            && cx.claim_binding_chord(TEXT, chord) == Some(ESC_CLOSE)
            && cx.layer_opening(ID) == Some(opening)
            && opening.layer_id() == cx.top_layer()
            && self
                .request
                .as_ref()
                .is_some_and(|r| r.is_current(workbench, route_visible))
        {
            cx.close_layer(ID, None);
            self.retiring = true;
            self.drawn.set(None);
            self.set_interactive(false);
            self.restore_focus(cx, workbench, route_visible, true);
            return response | Response::consumed() | Response::changed();
        }

        let ctrl_c_alias = cx.update_cause() == termrock::UpdateCause::Event
            && cx.command() == Some(super::CANCEL_OR_QUIT);
        let viewport_response = text_viewport().update(cx, &mut self.viewport, &viewport_lines);
        let mut copied = viewport_response
            .action_ref()
            .and_then(|action| match action {
                ViewportAction::Copy(text) => Some(text.clone()),
                _ => None,
            });
        response |= viewport_response.erase();
        if copied.is_none() && ctrl_c_alias {
            let mut text = String::new();
            if self.viewport.copy_into(&viewport_lines, &mut text) {
                copied = Some(text);
            }
        }

        let close_id = ID.part(Part::CLOSE);
        let close_response = if cx.area(ID).is_some_and(|a| a.width >= 16 && a.height >= 4) {
            Button::new(close_id, "×")
                .variant(termrock::Variant::QUIET)
                .focusability(termrock::Focusability::ClickOnly)
                .update(cx)
        } else {
            Response::ignored()
        };
        let close_requested = close_response.activated();
        response |= close_response.erase();
        if close_requested
            && cx.layer_opening(ID) == Some(opening)
            && opening.layer_id() == cx.top_layer()
        {
            cx.close_layer(ID, None);
            self.retiring = true;
            self.drawn.set(None);
            self.set_interactive(false);
            self.restore_focus(cx, workbench, route_visible, true);
            return response | Response::changed();
        }
        if let Some(text) = copied
            && cx.layer_opening(ID) == Some(opening)
            && opening.layer_id() == cx.top_layer()
            && self
                .request
                .as_ref()
                .is_some_and(|r| r.is_current(workbench, route_visible))
            && !self.retiring
        {
            cx.clipboard_copy(&text);
            response |= Response::changed();
        }
        response
    }

    fn service_lifecycle_only(
        &mut self,
        cx: &mut Cx<'_>,
        opening: LayerOpening,
        lines: &[ViewportLine<'_>],
    ) -> Response<()> {
        if cx.layer_opening(ID) != Some(opening) {
            return Response::ignored();
        }
        let close_id = ID.part(Part::CLOSE);
        let text_bucket = lifecycle_bucket(cx, TEXT);
        let close_bucket = lifecycle_bucket(cx, close_id);
        if text_bucket == LifecycleBucket::Other {
            diagnose(cx, TEXT);
        }
        if close_bucket == LifecycleBucket::Other {
            diagnose(cx, close_id);
        }
        if text_bucket == LifecycleBucket::Other || close_bucket == LifecycleBucket::Other {
            return Response::ignored();
        }
        let mut response = Response::ignored();
        if text_bucket == LifecycleBucket::Lifecycle {
            response |= text_viewport()
                .update(cx, &mut self.viewport, lines)
                .erase();
        }
        if close_bucket == LifecycleBucket::Lifecycle {
            response |= Button::new(close_id, "×")
                .variant(termrock::Variant::QUIET)
                .focusability(termrock::Focusability::ClickOnly)
                .disabled(true)
                .update(cx)
                .erase();
        }
        response
    }

    fn take_owned_receipts(&mut self, cx: &mut Cx<'_>, opening: LayerOpening) -> bool {
        let mut closed = false;
        while let Some(receipt) = cx.take_layer_intent_receipt_for(opening) {
            match receipt.event() {
                LayerEvent::Dismissed(_) | LayerEvent::Closed(_) => {
                    self.closed_receipt_seen = true;
                    closed = true;
                }
                LayerEvent::Opened => {}
            }
        }
        closed
    }

    fn close_when_top(
        &mut self,
        cx: &mut Cx<'_>,
        workbench: &Workbench,
        route_visible: bool,
        opening: LayerOpening,
        allow_focus: bool,
    ) -> Response<()> {
        self.set_interactive(false);
        self.drawn.set(None);
        if cx.layer_opening(ID) != Some(opening) || opening.layer_id() != cx.top_layer() {
            return Response::ignored();
        }
        cx.close_layer(ID, None);
        self.restore_focus(cx, workbench, route_visible, allow_focus);
        Response::changed()
    }

    fn restore_focus(
        &mut self,
        cx: &mut Cx<'_>,
        workbench: &Workbench,
        route_visible: bool,
        allow_focus: bool,
    ) {
        if self.focus_restored || !allow_focus {
            return;
        }
        let viewport = cx.viewport();
        let can_focus = viewport.width >= MIN_WIDTH
            && viewport.height >= MIN_HEIGHT
            && cx.top_layer() == LayerId::PAGE
            && self
                .request
                .as_ref()
                .is_some_and(|request| request.can_restore_focus(workbench, route_visible));
        if can_focus {
            cx.focus(super::HISTORY_LIST);
            self.focus_restored = true;
        }
    }

    fn set_interactive(&mut self, interactive: bool) -> bool {
        let changed = self.interactive != interactive;
        self.interactive = interactive;
        changed
    }

    fn clear_private(&mut self) {
        self.request = None;
        self.opening = None;
        self.retiring = false;
        self.closed_receipt_seen = false;
        self.focus_restored = false;
        self.interactive = false;
        self.drawn.set(None);
        self.state = DialogState::default();
        self.viewport = ViewportState::default();
    }

    pub(super) fn draw(&self, ui: &mut Ui<'_>, workbench: &Workbench, route_visible: bool) {
        self.drawn.set(None);
        let (Some(request), Some(opening)) = (self.request.as_ref(), self.opening) else {
            return;
        };
        let full = ui.full();
        if self.retiring
            || !request.is_current(workbench, route_visible)
            || full.width < MIN_WIDTH
            || full.height < MIN_HEIGHT
        {
            return;
        }
        let Some(dialog) = sized_dialog(&request.entry, full.width, full.height, ui.design())
        else {
            return;
        };
        let lines = request.text_lines();
        let viewport_lines: Vec<_> = lines
            .iter()
            .map(|line| ViewportLine::Plain(line.as_str()))
            .collect();
        let mut active_layer_drawn = false;
        let mut current_layer_area = None;
        let mut current_text_area = None;
        let _ = ui.layer_for(opening, |ui, area| {
            if ui.layer_id() != opening.layer_id() {
                return;
            }
            active_layer_drawn = !ui.is_inert() && !ui.is_covered_by_native_layer();
            current_layer_area = Some(area);
            if area.is_empty() {
                return;
            }
            current_text_area = dialog.draw(ui, area, &self.state, |ui, body| {
                if body.is_empty() {
                    return None;
                }
                let text_area = text_viewport().draw(ui, body, &self.viewport, &viewport_lines);
                if active_layer_drawn && self.viewport.selection().is_none() {
                    let flags = ui.state(TEXT);
                    ui.publish_additional_bindings(TEXT, flags, ESC_CLOSE_BINDINGS);
                }
                Some(text_area)
            });
        });
        if !active_layer_drawn {
            return;
        }
        let Some(layer_area) = current_layer_area else {
            return;
        };
        let Some(text_area) = current_text_area else {
            return;
        };
        let screen = termrock::Rect::new(0, 0, full.width, full.height);
        if layer_area.is_empty()
            || text_area.is_empty()
            || layer_area.intersection(screen) != layer_area
            || text_area.intersection(layer_area) != text_area
            || text_area.intersection(screen) != text_area
        {
            return;
        }
        self.drawn.set(Some(DrawWitness {
            opening,
            width: full.width,
            height: full.height,
        }));
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LifecycleBucket {
    Empty,
    Lifecycle,
    Other,
}

fn lifecycle_bucket(cx: &mut Cx<'_>, owner: Id) -> LifecycleBucket {
    let mut saw = false;
    let mut other = false;
    for intent in cx.intents(owner) {
        saw = true;
        if !matches!(
            intent,
            Intent::FocusIn { .. } | Intent::FocusOut { .. } | Intent::Cancel
        ) {
            other = true;
        }
    }
    if other {
        LifecycleBucket::Other
    } else if saw {
        LifecycleBucket::Lifecycle
    } else {
        LifecycleBucket::Empty
    }
}
fn diagnose(cx: &mut Cx<'_>, owner: Id) {
    cx.diagnostic(Diagnostic::UndeliveredIntent { owner });
}

fn text_viewport() -> TextViewport<'static> {
    TextViewport::new(TEXT).wrap(true).reserve_gutter(false)
}

fn project_detail(entry: &HistoryEntry) -> Vec<String> {
    let mut lines = vec![format!(
        "Source: {} · {}",
        source_label(entry.source),
        entry.when()
    )];
    lines.push(format!("Connection: {}", entry.connection));
    lines.push(format!("Database: {}", entry.database));
    lines.push(format!("Schema: {}", entry.schema));
    lines.push(format!(
        "Duration: {}",
        entry.duration_ms.map_or_else(
            || "Not recorded".to_owned(),
            |duration| format!("{duration} ms")
        )
    ));
    lines.push(format!(
        "Rows: {}",
        entry
            .rows
            .map_or_else(|| "Not recorded".to_owned(), |rows| rows.to_string())
    ));
    lines.push("SQL:".to_owned());
    lines.extend(entry.sql.split('\n').map(str::to_owned));
    match &entry.error {
        Some(error) => {
            lines.push("Error:".to_owned());
            lines.extend(error.split('\n').map(str::to_owned));
        }
        None => lines.push("Error: none".to_owned()),
    }
    lines
}

fn source_label(source: HistorySource) -> &'static str {
    source.label()
}

fn sized_dialog(
    entry: &HistoryEntry,
    viewport_width: u16,
    viewport_height: u16,
    design: &termrock::DesignTokens,
) -> Option<Dialog<'static>> {
    if viewport_width < MIN_WIDTH || viewport_height < MIN_HEIGHT {
        return None;
    }
    let desired_width = 88.min(viewport_width.saturating_sub(2));
    let mut dialog = Dialog::chrome(ID)
        .title("History detail")
        .description("Ctrl+C Copy selection · y Copy selection")
        .close_button(ActionKey::CANCEL)
        .width(desired_width)
        .body_rows(1);
    let available_height = viewport_height.saturating_sub(2);
    let base_height = dialog.measured_height(design);
    let max_body_rows = available_height
        .saturating_sub(base_height)
        .saturating_add(1)
        .max(1);
    let text_width = desired_width.saturating_sub(8).max(1);
    let content_rows = project_detail(entry).iter().fold(0u16, |rows, line| {
        rows.saturating_add(u16::try_from(wrap(line, text_width).len()).unwrap_or(u16::MAX))
    });
    dialog = dialog.body_rows(max_body_rows.min(content_rows.max(1)));
    Some(dialog)
}
