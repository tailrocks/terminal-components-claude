use std::sync::Weak;

use tablepro_demo::db::{Connection, Environment, SafeMode, Table};
use tablepro_domain::domain::ResultGrid;
use tablepro_domain::tabs::{GridView, QueryTab, Tab, TabKey};
use tablepro_sql::sql::{self, Decision, Risk, Statement};
use termrock::{
    Action, ActionKey, Chord, Cx, Dialog, DialogAction, DialogState, FrameRead, Id, KeyCode,
    KeyMap, KeyPhase, LayerOpening, Response, TextViewport, Ui, ViewportLine, ViewportState, wrap,
};

use super::{Surface, TableProApp};

pub(super) const ID: Id = Id::root("tablepro.query-safety");
const PREVIEW: Id = ID.sub("sql-preview");
const ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Confirm"),
];
const RUN_ACTIONS: [Action<'static>; 2] = [
    Action::quiet(ActionKey::CANCEL, "Cancel"),
    Action::new(ActionKey::CONFIRM, "Run query"),
];

#[derive(Clone, PartialEq)]
struct ResultSnapshot {
    columns: Vec<(String, tablepro_demo::db::ColType)>,
    model: ResultGrid,
    state: termrock::GridState,
}

impl From<&GridView> for ResultSnapshot {
    fn from(grid: &GridView) -> Self {
        Self {
            columns: grid.columns.clone(),
            model: grid.model.clone(),
            state: grid.state.clone(),
        }
    }
}

#[derive(Clone, PartialEq)]
struct QuerySnapshot {
    stored_text: String,
    editor_text: String,
    saved_text: String,
    error: Option<String>,
    last_status: Option<(String, bool, u32)>,
    result: Option<ResultSnapshot>,
}

impl From<&QueryTab> for QuerySnapshot {
    fn from(tab: &QueryTab) -> Self {
        Self {
            stored_text: tab.query.clone(),
            editor_text: tab.editor_state.editor().text().to_owned(),
            saved_text: tab.saved_text.clone(),
            error: tab.error.clone(),
            last_status: tab
                .last_status
                .as_ref()
                .map(|status| (status.summary.clone(), status.is_error, status.duration_ms)),
            result: tab.result.as_ref().map(ResultSnapshot::from),
        }
    }
}

#[derive(Clone, PartialEq)]
struct Evaluation {
    statement: Statement,
    decision: Decision,
    risk: Risk,
    resolved_table: Option<Table>,
    acknowledgement: Option<String>,
    acknowledgement_required: bool,
}

pub(super) struct QuerySafetyRequest {
    key: TabKey,
    generation: u64,
    owner: Weak<()>,
    connection: Connection,
    app_connection: Connection,
    schema: String,
    safe_mode: SafeMode,
    query: String,
    snapshot: QuerySnapshot,
    evaluation: Option<Evaluation>,
    return_surface: Surface,
}

impl core::fmt::Debug for QuerySafetyRequest {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QuerySafetyRequest")
            .field("key", &self.key)
            .field("generation", &self.generation)
            .field("query", &"[redacted]")
            .field("acknowledgement", &"[redacted]")
            .finish_non_exhaustive()
    }
}

impl QuerySafetyRequest {
    pub(super) fn capture(app: &TableProApp, key: TabKey, query: String) -> Option<Self> {
        let Tab::Query(tab) = app.workbench.tab(key)? else {
            return None;
        };
        let evaluation = evaluate(app, &query);
        Some(Self {
            key,
            generation: app.workbench.generation(key)?,
            owner: app.workbench.owner_token(),
            connection: app.workbench.connection.clone(),
            app_connection: app.connection.clone(),
            schema: app.workbench.current_schema().to_owned(),
            safe_mode: app.safe_mode,
            query,
            snapshot: QuerySnapshot::from(tab),
            evaluation,
            return_surface: app.surface,
        })
    }

    pub(super) fn key(&self) -> TabKey {
        self.key
    }

    pub(super) fn query(&self) -> &str {
        &self.query
    }

    pub(super) const fn return_surface(&self) -> Surface {
        self.return_surface
    }

    pub(super) fn needs_review(&self) -> bool {
        self.evaluation
            .as_ref()
            .is_some_and(|evaluation| matches!(evaluation.decision, Decision::Confirm { .. }))
    }

    fn acknowledgement_required(&self) -> bool {
        self.evaluation
            .as_ref()
            .is_some_and(|evaluation| evaluation.acknowledgement_required)
    }

    fn acknowledgement(&self) -> Option<&str> {
        self.evaluation
            .as_ref()
            .and_then(|evaluation| evaluation.acknowledgement.as_deref())
    }

    pub(super) fn valid_acknowledgement_target(&self) -> bool {
        !self.acknowledgement_required() || self.acknowledgement().is_some()
    }

    pub(super) fn is_current(&self, app: &TableProApp) -> bool {
        if !app.workbench.matches_owner(&self.owner)
            || app.workbench.generation(self.key) != Some(self.generation)
            || app.workbench.connection != self.connection
            || app.connection != self.app_connection
            || app.workbench.current_schema() != self.schema
            || app.safe_mode != self.safe_mode
        {
            return false;
        }
        let Some(Tab::Query(tab)) = app.workbench.tab(self.key) else {
            return false;
        };
        if QuerySnapshot::from(tab) != self.snapshot {
            return false;
        }
        let evaluation = evaluate(app, &self.query);
        evaluation == self.evaluation
    }

    pub(super) fn authorize(self, app: &TableProApp) -> Option<AuthorizedExecution> {
        (self.needs_review() && self.valid_acknowledgement_target() && self.is_current(app))
            .then_some(AuthorizedExecution { request: self })
    }

    fn dialog(
        &self,
        viewport_width: u16,
        viewport_height: u16,
        design: &termrock::DesignTokens,
    ) -> Dialog<'_> {
        let description = "Review the target facts and full SQL before continuing.";
        let desired_width = 88.min(viewport_width.saturating_sub(2).max(1));
        let mut dialog = if let Some(token) = self.acknowledgement() {
            Dialog::acknowledge(ID, "Review query request", token)
                .input_label("Type the exact target to authorize")
                .description(description)
                .actions(if self.is_select() {
                    &RUN_ACTIONS
                } else {
                    &ACTIONS
                })
                .cancel(ActionKey::CANCEL)
                .primary(ActionKey::CONFIRM)
        } else {
            Dialog::confirm(ID, "Review query request", description)
                .actions(if self.is_select() {
                    &RUN_ACTIONS
                } else {
                    &ACTIONS
                })
                .cancel(ActionKey::CANCEL)
                .primary(ActionKey::CONFIRM)
        };
        dialog = dialog.width(desired_width).body_rows(1);
        let available_height = viewport_height.saturating_sub(2);
        let base_height = dialog.measured_height(design);
        let max_body_rows = available_height
            .saturating_sub(base_height)
            .saturating_add(1)
            .max(1);
        let text_width = desired_width.saturating_sub(8).max(1);
        let content_rows = self.review_text().lines().fold(0u16, |rows, line| {
            rows.saturating_add(u16::try_from(wrap(line, text_width).len()).unwrap_or(u16::MAX))
        });
        dialog.body_rows(max_body_rows.min(content_rows.max(1)))
    }

    fn is_select(&self) -> bool {
        self.evaluation
            .as_ref()
            .is_some_and(|evaluation| matches!(&evaluation.statement, Statement::Select(_)))
    }

    fn review_text(&self) -> String {
        let mut lines = Vec::with_capacity(10);
        if let Some(evaluation) = &self.evaluation {
            let target = evaluation.resolved_table.as_ref().map_or_else(
                || {
                    format!(
                        "{} (unverified)",
                        evaluation.statement.target().unwrap_or("unknown")
                    )
                },
                Table::qualified,
            );
            let scope = reviewed_scope(&evaluation.statement);
            let risk = if evaluation.risk.summary.is_empty() {
                "No additional risk detail".to_owned()
            } else {
                evaluation.risk.summary.clone()
            };
            lines.push(format!("Connection: {}", self.connection.name));
            lines.push(format!(
                "Environment: {} · database: {} · schema context: {}",
                self.connection.environment.label(),
                self.connection.database,
                self.schema
            ));
            lines.push(format!("Target: {target}"));
            if let Some(token) = &evaluation.acknowledgement {
                lines.push(format!("Acknowledgement: type exactly {token}"));
            }
            lines.push(format!(
                "Action: {} · scope: {scope}",
                evaluation.risk.action
            ));
            lines.push(format!("Risk: {risk}"));
            if !evaluation.risk.reversible.is_empty() {
                lines.push(format!("Reversibility: {}", evaluation.risk.reversible));
            }
            lines.push(format!("Safe mode: {}", self.safe_mode.label()));
        }
        lines.push(
            "unsupported demo executor: SELECT only; writes will be rejected after confirmation."
                .to_owned(),
        );
        lines.push("SQL:".to_owned());
        lines.extend(self.query.lines().map(str::to_owned));
        lines.join("\n")
    }
}

pub(super) struct AuthorizedExecution {
    request: QuerySafetyRequest,
}

impl AuthorizedExecution {
    pub(super) fn into_request(self) -> QuerySafetyRequest {
        self.request
    }
}

#[derive(Default)]
pub(super) struct QuerySafetyController {
    request: Option<QuerySafetyRequest>,
    opening: Option<LayerOpening>,
    escape_capture_installed: bool,
    state: DialogState,
    preview_state: ViewportState,
}

impl core::fmt::Debug for QuerySafetyController {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("QuerySafetyController")
            .field("open", &self.request.is_some())
            .field("request", &self.request)
            .finish_non_exhaustive()
    }
}

impl QuerySafetyController {
    pub(super) fn is_open(&self) -> bool {
        self.request.is_some()
    }

    pub(super) fn open(&mut self, cx: &mut Cx<'_>, request: QuerySafetyRequest) -> bool {
        if self.request.is_some() || !request.valid_acknowledgement_target() {
            return false;
        }
        let screen = cx.viewport();
        let dialog = request.dialog(screen.width, screen.height, cx.design());
        let focus = if request.acknowledgement().is_some() {
            dialog.input_id()
        } else {
            dialog.action_id(0)
        };
        let Some(opening) = cx.open_layer_with_token(ID, dialog.layer(cx).initial_focus(focus))
        else {
            return false;
        };
        self.opening = Some(opening);
        self.preview_state = ViewportState::default();
        self.preview_state.set_follow(false);
        self.state = DialogState::default();
        self.request = Some(request);
        true
    }

    pub(super) fn install_escape_capture(&mut self, keymap: &mut KeyMap) {
        if !self.escape_capture_installed {
            keymap.add(
                KeyPhase::Capture,
                Chord::key(KeyCode::Esc),
                super::CANCEL_QUERY_SAFETY,
            );
            self.escape_capture_installed = true;
        }
    }

    fn sync_escape_capture(&mut self, keymap: &mut KeyMap, active_opening: bool) {
        if active_opening && !self.escape_capture_installed {
            self.install_escape_capture(keymap);
        } else if !active_opening && self.escape_capture_installed {
            keymap.remove(KeyPhase::Capture, Chord::key(KeyCode::Esc));
            self.escape_capture_installed = false;
        }
    }

    pub(super) fn sync_layer_scope(&mut self, keymap: &mut KeyMap, cx: &Cx<'_>) {
        let active_opening = self.request.is_some()
            && self.opening.is_some_and(|opening| {
                cx.layer_opening(ID) == Some(opening) && cx.top_layer() == opening.layer_id()
            });
        self.sync_escape_capture(keymap, active_opening);
    }

    pub(super) fn update(&mut self, app: &mut TableProApp, cx: &mut Cx<'_>) -> Response<()> {
        let had_request = self.request.is_some();
        let opening_matches = self
            .opening
            .is_some_and(|opening| cx.layer_opening(ID) == Some(opening));
        let active_opening = had_request
            && opening_matches
            && self
                .opening
                .is_some_and(|opening| cx.top_layer() == opening.layer_id());
        self.sync_escape_capture(&mut app.keymap, active_opening);
        if had_request && cx.is_open(ID) && !opening_matches {
            if let Some(request) = self.request.take()
                && app.surface == Surface::SafetyDialogTypedAck
            {
                app.surface = request.return_surface();
            }
            self.opening = None;
            self.state = DialogState::default();
            self.preview_state = ViewportState::default();
            return Response::ignored();
        }
        let escape_cancel = active_opening && cx.command() == Some(super::CANCEL_QUERY_SAFETY);
        if escape_cancel {
            cx.close_layer(ID, None);
        }
        let screen = cx.viewport();
        let fallback = Dialog::confirm(ID, "", "");
        let dialog = self.request.as_ref().map_or_else(
            || {
                fallback
                    .width(screen.width.saturating_sub(2).max(1))
                    .body_rows(0)
            },
            |request| request.dialog(screen.width, screen.height, cx.design()),
        );
        let review_text = self.request.as_ref().map(QuerySafetyRequest::review_text);
        let lines: Vec<_> = review_text.as_deref().map_or_else(Vec::new, |text| {
            text.split('\n').map(ViewportLine::Plain).collect()
        });
        let scroll_response = TextViewport::new(PREVIEW)
            .wrap(true)
            .reserve_gutter(false)
            .update(cx, &mut self.preview_state, &lines)
            .erase();
        let response = dialog.update(cx, &mut self.state);
        let action = response.action_ref().copied();
        let confirmed = active_opening
            && cx.is_open(ID)
            && matches!(action, Some(DialogAction::Action(ActionKey::CONFIRM)));
        let dismissed = active_opening
            && (escape_cancel
                || matches!(
                    action,
                    Some(DialogAction::Dismissed(_) | DialogAction::Action(ActionKey::CANCEL))
                )
                || !cx.is_open(ID));

        let stale_opening = had_request && !opening_matches;

        if confirmed || dismissed || stale_opening {
            if (confirmed || dismissed)
                && let Some(opening) = self.opening
                && cx.layer_opening(ID) == Some(opening)
            {
                cx.close_layer(ID, None);
            }
            if let Some(request) = self.request.take() {
                let key = request.key;
                let owner = request.owner.clone();
                if app.surface == Surface::SafetyDialogTypedAck {
                    app.surface = request.return_surface();
                }
                if confirmed {
                    if let Some(authorized) = request.authorize(app) {
                        let _ = app.execute_authorized_query(authorized);
                    } else {
                        app.status = "Work changed; request again".to_owned();
                    }
                }
                if confirmed || dismissed {
                    focus_request_owner(app, cx, key, &owner);
                }
            }
            self.opening = None;
            self.sync_escape_capture(&mut app.keymap, false);
            self.state = DialogState::default();
            self.preview_state = ViewportState::default();
        }
        response.erase() | scroll_response
    }

    pub(super) fn draw(&self, ui: &mut Ui<'_>) {
        let Some(request) = self.request.as_ref() else {
            return;
        };
        let full = ui.full();
        let dialog = request.dialog(full.width, full.height, ui.design());
        let text = request.review_text();
        let lines: Vec<_> = text.split('\n').map(ViewportLine::Plain).collect();
        ui.layer(ID, |ui, area| {
            dialog.draw(ui, area, &self.state, |ui, body| {
                TextViewport::new(PREVIEW)
                    .wrap(true)
                    .reserve_gutter(false)
                    .draw(ui, body, &self.preview_state, &lines);
            });
        });
    }
}

fn focus_request_owner(app: &mut TableProApp, cx: &mut Cx<'_>, key: TabKey, owner: &Weak<()>) {
    if app.workbench.matches_owner(owner) && matches!(app.workbench.tab(key), Some(Tab::Query(_))) {
        if app.workbench.active_key() != Some(key) {
            let _ = app.workbench.activate(key);
            app.sync_active_tab();
        }
        cx.focus(key.control("query"));
    } else if let Some(focus) = app.query_id().or_else(|| app.result_id()) {
        cx.focus(focus);
    } else {
        cx.focus(super::EXPLORER);
    }
}

fn evaluate(app: &TableProApp, query: &str) -> Option<Evaluation> {
    let statement = sql::parse(query.trim()).ok()?;
    let resolved_table = resolve_table(app, &statement);
    let risk = sql::assess(&statement, resolved_table.as_ref());
    let decision = sql::gate(app.safe_mode, &statement);
    let acknowledgement_required = matches!(decision, Decision::Confirm { deliberate: true })
        || (app.workbench.connection.environment == Environment::Production && risk.dangerous);
    let acknowledgement = if acknowledgement_required {
        statement.target().map(str::to_owned)
    } else {
        None
    };
    Some(Evaluation {
        statement,
        decision,
        risk,
        resolved_table,
        acknowledgement,
        acknowledgement_required,
    })
}

fn resolve_table(app: &TableProApp, statement: &Statement) -> Option<Table> {
    let Statement::Select(select) = statement else {
        return None;
    };
    app.catalog
        .find(select.schema.as_deref(), &select.table)
        .cloned()
}

fn reviewed_scope(statement: &Statement) -> String {
    match statement {
        Statement::Select(_) => "rows selected by this query (preview only)".to_owned(),
        Statement::Update {
            table,
            has_where: true,
        }
        | Statement::Delete {
            table,
            has_where: true,
        } => format!("rows matching WHERE in {table}; predicate is not evaluated here"),
        Statement::Update {
            table,
            has_where: false,
        }
        | Statement::Delete {
            table,
            has_where: false,
        }
        | Statement::Truncate { table } => format!("all rows in {table}"),
        Statement::Insert { table } => format!("a new row in {table}"),
        Statement::Drop { name, .. } | Statement::Create { name, .. } => name.clone(),
        Statement::Alter { table, .. } => format!("the structure of {table}"),
        Statement::Explain { inner, .. } => reviewed_scope(inner),
        Statement::Other(_) => "unverified statement scope".to_owned(),
    }
}
