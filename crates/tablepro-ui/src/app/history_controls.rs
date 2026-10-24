use termrock::{
    BlurPolicy, Button, Cx, Diagnostic, FrameRead, Intent, Response, StateFlags, TextAction,
    TextInput, TextInputState, Ui,
};

use crate::tabs::TabKey;

const SEARCH_CONTROL: &str = "history.search";
const CONNECTION_CONTROL: &str = "history.connection";
const STATUS_CONTROL: &str = "history.status";

const CONNECTION_WIDTH: u16 = 22;
const STATUS_WIDTH: u16 = 15;
const TOOLBAR_GAP: u16 = 1;
const MIN_SEARCH_WIDTH: u16 = 24;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum ConnectionScope {
    #[default]
    All,
    Current,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum IntentBucket {
    Empty,
    FocusOnly,
    Other,
}

fn classify(cx: &mut Cx<'_>, owner: termrock::Id) -> IntentBucket {
    let mut saw_intent = false;
    let mut saw_other = false;
    for intent in cx.intents(owner) {
        saw_intent = true;
        if !matches!(intent, Intent::FocusIn { .. } | Intent::FocusOut { .. }) {
            saw_other = true;
        }
    }
    if saw_other {
        IntentBucket::Other
    } else if saw_intent {
        IntentBucket::FocusOnly
    } else {
        IntentBucket::Empty
    }
}

fn diagnose_unexpected(cx: &mut Cx<'_>, owner: termrock::Id) {
    cx.diagnostic(Diagnostic::UndeliveredIntent { owner });
}

pub(super) struct HistoryControls {
    search: TextInputState,
    scope: ConnectionScope,
    failed_only: bool,
    last_connection: Option<String>,
    committed_search: String,
    initialized: bool,
}

impl Default for HistoryControls {
    fn default() -> Self {
        Self {
            search: TextInputState::default(),
            scope: ConnectionScope::All,
            failed_only: false,
            last_connection: None,
            committed_search: String::new(),
            initialized: false,
        }
    }
}

impl HistoryControls {
    pub(super) fn scope(&self) -> ConnectionScope {
        self.scope
    }

    pub(super) const fn failed_only(&self) -> bool {
        self.failed_only
    }

    pub(super) fn update(
        &mut self,
        cx: &mut Cx<'_>,
        key: TabKey,
        committed_search: &mut String,
        current_connection: &str,
        active: bool,
    ) -> (Response<()>, bool, bool) {
        let mut refresh = false;
        let mut response = Response::ignored();
        let mut inactive_buckets_safe = true;

        let search_id = key.control(SEARCH_CONTROL);
        if active {
            let search = TextInput::new(search_id)
                .placeholder("SQL terms · Enter applies, Esc cancels")
                .blur(BlurPolicy::Keep)
                .update(cx, &mut self.search, committed_search);
            if matches!(search.action_ref(), Some(TextAction::Committed)) {
                refresh = true;
            }
            response |= search.erase();
        } else {
            match classify(cx, search_id) {
                IntentBucket::Other => {
                    diagnose_unexpected(cx, search_id);
                    inactive_buckets_safe = false;
                }
                IntentBucket::Empty | IntentBucket::FocusOnly => {
                    // Hidden lifecycle may settle focus, but it must not commit into the tab model.
                    let mut retained_search = committed_search.clone();
                    response |= TextInput::new(search_id)
                        .placeholder("SQL terms · Enter applies, Esc cancels")
                        .blur(BlurPolicy::Keep)
                        .update(cx, &mut self.search, &mut retained_search)
                        .erase();
                }
            }
        }

        let scope_label = match self.scope {
            ConnectionScope::All => "All connections",
            ConnectionScope::Current => "Current connection",
        };
        let scope_id = key.control(CONNECTION_CONTROL);
        if active {
            let scope = Button::new(scope_id, scope_label)
                .checked(self.scope == ConnectionScope::Current)
                .update(cx);
            let scope_activated = scope.activated();
            response |= scope.erase();
            if scope_activated {
                self.scope = match self.scope {
                    ConnectionScope::All => ConnectionScope::Current,
                    ConnectionScope::Current => ConnectionScope::All,
                };
                refresh = true;
            }
        } else if let Some(button_response) = update_inactive_button(
            cx,
            scope_id,
            scope_label,
            self.scope == ConnectionScope::Current,
        ) {
            response |= button_response;
        } else {
            inactive_buckets_safe = false;
        }

        let status_label = if self.failed_only {
            "Failed only"
        } else {
            "Any status"
        };
        let status_id = key.control(STATUS_CONTROL);
        if active {
            let status = Button::new(status_id, status_label)
                .checked(self.failed_only)
                .update(cx);
            let status_activated = status.activated();
            response |= status.erase();
            if status_activated {
                self.failed_only = !self.failed_only;
                refresh = true;
            }
        } else if let Some(button_response) =
            update_inactive_button(cx, status_id, status_label, self.failed_only)
        {
            response |= button_response;
        } else {
            inactive_buckets_safe = false;
        }

        if active {
            let connection_changed = self.last_connection.as_deref() != Some(current_connection);
            if !self.initialized || (self.scope == ConnectionScope::Current && connection_changed) {
                refresh = true;
            }
            self.initialized = true;
            if connection_changed {
                self.last_connection = Some(current_connection.to_owned());
            }
        }

        if self.committed_search.as_str() != committed_search.as_str() {
            self.committed_search.clone_from(committed_search);
        }
        (response, refresh, inactive_buckets_safe)
    }

    pub(super) fn service_orphan(&mut self, cx: &mut Cx<'_>, key: TabKey) -> (Response<()>, bool) {
        let mut retained_search = self.committed_search.clone();
        let (response, _, buckets_safe) = self.update(cx, key, &mut retained_search, "", false);
        let owners_retired = [
            key.control(SEARCH_CONTROL),
            key.control(CONNECTION_CONTROL),
            key.control(STATUS_CONTROL),
        ]
        .into_iter()
        .all(|owner| {
            cx.area(owner).is_none()
                && !cx.state(owner).contains(StateFlags::FOCUSED)
                && cx.capture_owner() != Some(owner)
        });
        (response, buckets_safe && owners_retired)
    }

    pub(super) fn mark_refreshed(&mut self, committed_search: &str, current_connection: &str) {
        self.initialized = true;
        if self.last_connection.as_deref() != Some(current_connection) {
            self.last_connection = Some(current_connection.to_owned());
        }
        if self.committed_search.as_str() != committed_search {
            self.committed_search.clear();
            self.committed_search.push_str(committed_search);
        }
    }

    pub(super) fn draw(
        &self,
        ui: &mut Ui<'_>,
        inner: termrock::Rect,
        key: TabKey,
        committed_search: &str,
    ) -> ToolbarLayout {
        let layout = self.layout(inner);
        let _ = TextInput::new(key.control(SEARCH_CONTROL))
            .value(committed_search)
            .placeholder("SQL terms · Enter applies, Esc cancels")
            .blur(BlurPolicy::Keep)
            .draw(ui, layout.search, &self.search);

        let scope_label = match self.scope {
            ConnectionScope::All => "All connections",
            ConnectionScope::Current => "Current connection",
        };
        let _ = Button::new(key.control(CONNECTION_CONTROL), scope_label)
            .checked(self.scope == ConnectionScope::Current)
            .draw(ui, layout.connection);

        let status_label = if self.failed_only {
            "Failed only"
        } else {
            "Any status"
        };
        let _ = Button::new(key.control(STATUS_CONTROL), status_label)
            .checked(self.failed_only)
            .draw(ui, layout.status);
        layout
    }

    pub(super) fn layout(&self, inner: termrock::Rect) -> ToolbarLayout {
        ToolbarLayout::new(inner)
    }
}

fn update_inactive_button(
    cx: &mut Cx<'_>,
    owner: termrock::Id,
    label: &str,
    checked: bool,
) -> Option<Response<()>> {
    match classify(cx, owner) {
        IntentBucket::Other => {
            diagnose_unexpected(cx, owner);
            None
        }
        IntentBucket::Empty | IntentBucket::FocusOnly => Some(
            Button::new(owner, label)
                .checked(checked)
                .update(cx)
                .erase(),
        ),
    }
}

#[derive(Clone, Copy)]
pub(super) struct ToolbarLayout {
    pub(super) search: termrock::Rect,
    pub(super) connection: termrock::Rect,
    pub(super) status: termrock::Rect,
    pub(super) list_y: u16,
}

impl ToolbarLayout {
    fn new(inner: termrock::Rect) -> Self {
        let button_widths = CONNECTION_WIDTH
            .saturating_add(STATUS_WIDTH)
            .saturating_add(2 * TOOLBAR_GAP);
        let horizontal_search_width = inner.width.saturating_sub(button_widths);
        if horizontal_search_width >= MIN_SEARCH_WIDTH {
            let search = row(inner, inner.x, 1, horizontal_search_width);
            let connection_x = search
                .x
                .saturating_add(search.width)
                .saturating_add(TOOLBAR_GAP);
            let connection = row(inner, connection_x, 1, CONNECTION_WIDTH);
            let status_x = connection
                .x
                .saturating_add(connection.width)
                .saturating_add(TOOLBAR_GAP);
            let status = row(inner, status_x, 1, STATUS_WIDTH);
            Self {
                search,
                connection,
                status,
                list_y: inner.y.saturating_add(3).min(inner.bottom()),
            }
        } else {
            let search = row(inner, inner.x, 1, inner.width);
            let connection_width = inner.width.saturating_sub(TOOLBAR_GAP) / 2;
            let connection = row(inner, inner.x, 2, connection_width);
            let status_x = inner
                .x
                .saturating_add(connection_width)
                .saturating_add(TOOLBAR_GAP);
            let status_width = inner.width.saturating_sub(connection_width + TOOLBAR_GAP);
            let status = row(inner, status_x, 2, status_width);
            Self {
                search,
                connection,
                status,
                list_y: inner.y.saturating_add(4).min(inner.bottom()),
            }
        }
    }
}

fn row(inner: termrock::Rect, x: u16, y_offset: u16, width: u16) -> termrock::Rect {
    let right = inner.x.saturating_add(inner.width);
    let bottom = inner.y.saturating_add(inner.height);
    let y = inner.y.saturating_add(y_offset);
    termrock::Rect::new(
        x,
        y,
        right.saturating_sub(x).min(width),
        u16::from(y < bottom),
    )
}
