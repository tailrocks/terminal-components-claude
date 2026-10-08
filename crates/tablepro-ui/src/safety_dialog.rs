use termrock::{
    Action, ActionKey, Cx, DesignTokens, Dialog, DialogAction, DialogState, FgStep, FrameRead, Id,
    Intent, ItemKey, Part, Props, PropsRow, Rect, Response, Role, Ui, wrap,
};

pub const SAFETY_DIALOG: Id = Id::root("tablepro.safety-dialog");
pub const SAFETY_INPUT: Id = SAFETY_DIALOG.part(Part::FIELD);
pub const SAFETY_CANCEL: Id = SAFETY_DIALOG.part(Part::ACTIONS).index(0);
pub const SAFETY_CONFIRM: Id = SAFETY_DIALOG.part(Part::ACTIONS).index(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Normal,
    Secondary,
    Warning,
    Error,
    Muted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Prop {
    pub label: String,
    pub value: String,
    pub tone: Tone,
    pub wrap: bool,
}

impl Prop {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            tone: Tone::Normal,
            wrap: false,
        }
    }

    pub fn tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.wrap = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyFocus {
    Input,
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyDialogAction {
    Cancel,
    Confirm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyIntent {
    Query,
    Commit,
}

const QUERY_TOKEN_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::primary(ActionKey::CONFIRM, "Execute"),
];
const COMMIT_TOKEN_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::primary(ActionKey::CONFIRM, "Save"),
];
const DANGER_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::danger(ActionKey::CONFIRM, "Delete"),
];
const QUERY_PLAIN_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::primary(ActionKey::CONFIRM, "Execute"),
];
const COMMIT_PLAIN_ACTIONS: [Action<'static>; 2] = [
    Action::new(ActionKey::CANCEL, "Cancel"),
    Action::primary(ActionKey::CONFIRM, "Save"),
];

fn tone_role(tone: Tone) -> Role {
    match tone {
        Tone::Normal => Role::Fg(FgStep::Primary),
        Tone::Secondary => Role::Fg(FgStep::Secondary),
        Tone::Warning => Role::Warning,
        Tone::Error => Role::Danger,
        Tone::Muted => Role::Fg(FgStep::Muted),
    }
}

#[derive(Debug, Clone)]
pub struct SafetyDialog {
    pub id: Id,
    pub title: String,
    pub facts: Vec<Prop>,
    pub code: Vec<String>,
    pub token: Option<String>,
    pub confirm_label: String,
    pub confirm_danger: bool,
    pub width: u16,
    pub focus: SafetyFocus,
    pub intent: SafetyIntent,
    state: DialogState,
    max_height: Option<u16>,
}

impl SafetyDialog {
    #[expect(clippy::too_many_arguments)]
    pub fn new(
        title: impl Into<String>,
        facts: Vec<Prop>,
        code: Vec<String>,
        token: Option<String>,
        confirm_label: impl Into<String>,
        confirm_danger: bool,
        width: u16,
        intent: SafetyIntent,
    ) -> Self {
        let tok = token;
        let initial_focus = if tok.is_some() {
            SafetyFocus::Input
        } else if confirm_danger {
            SafetyFocus::Cancel
        } else {
            SafetyFocus::Confirm
        };
        Self {
            id: Id::root("tablepro.safety-dialog"),
            title: title.into(),
            facts,
            code,
            token: tok,
            confirm_label: confirm_label.into(),
            confirm_danger,
            width,
            focus: initial_focus,
            intent,
            state: DialogState::default(),
            max_height: None,
        }
    }

    pub fn armed(&self) -> bool {
        match &self.token {
            Some(tok) => Dialog::acknowledge(SAFETY_DIALOG, &self.title, tok).armed(&self.state),
            None => true,
        }
    }

    pub fn is_editing(&self) -> bool {
        self.state.is_editing()
    }

    /// The layer-height clamp, applied by the open sites (which own the
    /// screen rows) and reused on every frame so D1 re-asserts the size
    /// the layer opened with.
    pub fn set_max_height(&mut self, h: u16) {
        self.max_height = Some(h);
    }

    fn kind_actions(token: bool, danger: bool, intent: SafetyIntent) -> &'static [Action<'static>] {
        match (token, danger, intent) {
            (true, false, SafetyIntent::Query) => &QUERY_TOKEN_ACTIONS,
            (true, false, SafetyIntent::Commit) => &COMMIT_TOKEN_ACTIONS,
            (true, true, _) => &DANGER_ACTIONS,
            (false, false, SafetyIntent::Query) => &QUERY_PLAIN_ACTIONS,
            (false, false, SafetyIntent::Commit) => &COMMIT_PLAIN_ACTIONS,
            (false, true, _) => &DANGER_ACTIONS,
        }
    }

    /// Wrapped fact rows over the dialog's own content width, so the
    /// anchored code block lands exactly on its sequential position.
    fn count_rows(width: u16, facts: &[Prop], d: &DesignTokens) -> u16 {
        let inner = Dialog::new(SAFETY_DIALOG).width(width).inner_width(d);
        let label_w = facts
            .iter()
            .map(|p| p.label.len())
            .max()
            .unwrap_or(0)
            .min(usize::from(u16::MAX)) as u16;
        let vw = inner.saturating_sub(label_w).saturating_sub(2).max(4);
        let mut rows = 0u16;
        for p in facts {
            if p.wrap {
                rows =
                    rows.saturating_add(wrap(&p.value, vw).len().min(usize::from(u16::MAX)) as u16);
            } else {
                rows = rows.saturating_add(1);
            }
        }
        rows
    }

    /// The stock dialog for this model. Token kinds take the typed
    /// acknowledgement below the body with the idle gate and plaintext
    /// echo; plain kinds take no input and focus the primary action
    /// (or Cancel for danger). Never a description: y/n must type text.
    #[expect(
        clippy::too_many_arguments,
        reason = "split borrows keep state mutable"
    )]
    fn build<'s>(
        title: &'s str,
        codes: &'s [&'s str],
        token: &'s Option<String>,
        confirm_danger: bool,
        width: u16,
        intent: SafetyIntent,
        max_height: Option<u16>,
        label: Option<&'s str>,
        facts_rows: u16,
    ) -> Dialog<'s> {
        let actions = Self::kind_actions(token.is_some(), confirm_danger, intent);
        let base = match token.as_deref() {
            Some(tok) => Dialog::acknowledge(SAFETY_DIALOG, title, tok)
                .actions(actions)
                .input_after_body(true)
                .idle_ack(true)
                .ack_secret(false),
            None => {
                let plain = Dialog::new(SAFETY_DIALOG)
                    .title(title)
                    .actions(actions)
                    .cancel(ActionKey::CANCEL);
                if confirm_danger {
                    plain
                } else {
                    plain.primary(ActionKey::CONFIRM)
                }
            }
        };
        let mut dlg = base.width(width).body_rows(facts_rows).code(codes);
        if let Some(max) = max_height {
            dlg = dlg.max_height(max);
        }
        if let Some(l) = label {
            dlg = dlg.input_label(l);
        }
        dlg
    }

    /// Open the modal layer for this dialog. Initial focus comes from
    /// the stock dialog (input, primary, or Cancel by kind).
    pub fn open_layer(&self, cx: &mut Cx<'_>) {
        let codes: Vec<&str> = self.code.iter().map(String::as_str).collect();
        let label = self
            .token
            .as_ref()
            .map(|tok| format!("Type {tok} to confirm"));
        let facts_rows = Self::count_rows(self.width, &self.facts, cx.design());
        let dlg = Self::build(
            &self.title,
            &codes,
            &self.token,
            self.confirm_danger,
            self.width,
            self.intent,
            self.max_height,
            label.as_deref(),
            facts_rows,
        );
        let mut spec = dlg.layer(cx);
        spec.restore_focus = true;
        cx.open_layer(SAFETY_DIALOG, spec);
    }

    /// Host the stock dialog: mirror focus into the model, then drive.
    pub fn update(&mut self, cx: &mut Cx<'_>) -> Response<DialogAction> {
        for (id, focus) in [
            (SAFETY_INPUT, SafetyFocus::Input),
            (SAFETY_CANCEL, SafetyFocus::Cancel),
            (SAFETY_CONFIRM, SafetyFocus::Confirm),
        ] {
            for intent in cx.intents(id) {
                if let Intent::FocusIn { .. } = intent {
                    self.focus = focus;
                }
            }
        }
        let codes: Vec<&str> = self.code.iter().map(String::as_str).collect();
        let label = self
            .token
            .as_ref()
            .map(|tok| format!("Type {tok} to confirm"));
        let facts_rows = Self::count_rows(self.width, &self.facts, cx.design());
        let Self {
            title,
            token,
            confirm_danger,
            width,
            intent,
            max_height,
            state,
            ..
        } = self;
        let dlg = Self::build(
            title,
            &codes,
            token,
            *confirm_danger,
            *width,
            *intent,
            *max_height,
            label.as_deref(),
            facts_rows,
        );
        dlg.update(cx, state)
    }

    pub fn draw(&self, ui: &mut Ui<'_>, area: Rect) {
        let codes: Vec<&str> = self.code.iter().map(String::as_str).collect();
        let label = self
            .token
            .as_ref()
            .map(|tok| format!("Type {tok} to confirm"));
        let facts_rows = Self::count_rows(self.width, &self.facts, ui.design());
        let dlg = Self::build(
            &self.title,
            &codes,
            &self.token,
            self.confirm_danger,
            self.width,
            self.intent,
            self.max_height,
            label.as_deref(),
            facts_rows,
        );
        let rows: Vec<PropsRow<'_>> = self
            .facts
            .iter()
            .enumerate()
            .map(|(i, p)| {
                PropsRow::new(ItemKey::index(i), &p.label, &p.value)
                    .tone(tone_role(p.tone))
                    .wrap_if(p.wrap)
            })
            .collect();
        dlg.draw(ui, area, &self.state, |ui, page| {
            Props::rich(&rows).draw(ui, page);
        });
    }
}
