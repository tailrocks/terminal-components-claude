use termrock::{
    Action, ActionKey, EnterPolicy, FgStep, FieldKind, FieldMut, FieldRef, FieldSizingPolicy,
    FieldSpec, FormData, FormDialog, FormDialogSubmitPolicy, Id, Part, Role,
};

pub(super) const DIALOG_ID: Id = Id::root("showcase_ui::pages::query_workbench.options");
const FORM_ID: Id = DIALOG_ID.part(Part::BODY);
const ALLOW_NEW: Id = FORM_ID.part(Part::FIELD).index(0);
const CLOSE_REVEAL: Id = FORM_ID.part(Part::FIELD).index(1);
const READ_ONLY: Id = FORM_ID.part(Part::FIELD).index(2);
const RESULT: Id = FORM_ID.part(Part::FIELD).index(3);
const ENVIRONMENT: Id = FORM_ID.part(Part::FIELD).index(4);
const ENVIRONMENT_NOTE: Id = FORM_ID.part(Part::FIELD).index(5);
const EDITOR_SHARE: Id = FORM_ID.part(Part::FIELD).index(6);
const SHOW_EXPLORER: Id = FORM_ID.part(Part::FIELD).index(7);
const EXPLORER_SHARE: Id = FORM_ID.part(Part::FIELD).index(8);

pub(super) const ALLOW_NEW_KEY: &str = "queryworkbench.allow_new";
pub(super) const CLOSE_REVEAL_KEY: &str = "queryworkbench.close_reveal";
pub(super) const READ_ONLY_KEY: &str = "queryworkbench.read_only";
pub(super) const RESULT_KEY: &str = "queryworkbench.result";
pub(super) const ENVIRONMENT_KEY: &str = "queryworkbench.env";
pub(super) const EDITOR_SHARE_KEY: &str = "queryworkbench.editor_share";
pub(super) const SHOW_EXPLORER_KEY: &str = "queryworkbench.explorer";
pub(super) const EXPLORER_SHARE_KEY: &str = "queryworkbench.explorer_width";

const CLOSE: ActionKey = ActionKey::CANCEL;
const ACTIONS: &[Action<'static>] = &[Action::quiet(CLOSE, "Close")];
const CLOSE_REVEAL_OPTIONS: &[&str] = &["always", "hover"];
const RESULT_OPTIONS: &[&str] = &["run", "empty", "error", "plan"];
const ENVIRONMENT_OPTIONS: &[&str] = &["production", "staging", "development", "local"];
const EDITOR_SHARE_OPTIONS: &[&str] = &["40", "25", "60"];
const EXPLORER_SHARE_OPTIONS: &[&str] = &["32", "24", "40"];
const ENVIRONMENT_ROWS: [(&str, Role); 5] = [
    ("Sample environments", Role::Info),
    ("Production: sample data only", Role::Danger),
    ("Staging: sample data only", Role::Warning),
    ("Development: sample data only", Role::Info),
    ("Local: sample data only", Role::Fg(FgStep::Muted)),
];
const FIELDS: &[FieldSpec<'static>] = &[
    FieldSpec::new(
        ALLOW_NEW,
        "",
        FieldKind::Toggle(termrock::Toggle::new(ALLOW_NEW, "Allow new query tabs")),
    ),
    FieldSpec::new(
        CLOSE_REVEAL,
        "Tab close buttons",
        FieldKind::Select(termrock::Select::new(CLOSE_REVEAL)),
    )
    .help("Show close buttons always or while a tab is in use."),
    FieldSpec::new(
        READ_ONLY,
        "",
        FieldKind::Toggle(termrock::Toggle::new(READ_ONLY, "Read-only SQL input")),
    ),
    FieldSpec::new(
        RESULT,
        "Result fixture",
        FieldKind::Select(termrock::Select::new(RESULT)),
    )
    .help("Apply one sample result to the active query document."),
    FieldSpec::new(
        ENVIRONMENT,
        "Environment",
        FieldKind::Select(termrock::Select::new(ENVIRONMENT)),
    )
    .help("Sample labels only; this does not grant database permissions."),
    FieldSpec::new(ENVIRONMENT_NOTE, "", FieldKind::Note),
    FieldSpec::new(
        EDITOR_SHARE,
        "Editor %",
        FieldKind::Select(termrock::Select::new(EDITOR_SHARE)),
    )
    .help("Choose the editor pane height percentage; Home resets to this preset."),
    FieldSpec::new(
        SHOW_EXPLORER,
        "",
        FieldKind::Toggle(termrock::Toggle::new(SHOW_EXPLORER, "Show Explorer")),
    ),
    FieldSpec::new(
        EXPLORER_SHARE,
        "Explorer %",
        FieldKind::Select(termrock::Select::new(EXPLORER_SHARE)),
    )
    .help("Choose the Explorer pane width percentage; Home resets to this preset."),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ResultFixture {
    Run,
    Empty,
    Error,
    Plan,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CloseReveal {
    Always,
    Hover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditorShare {
    Forty,
    TwentyFive,
    Sixty,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ExplorerShare {
    ThirtyTwo,
    TwentyFour,
    Forty,
}

impl ExplorerShare {
    const fn index(self) -> usize {
        match self {
            Self::ThirtyTwo => 0,
            Self::TwentyFour => 1,
            Self::Forty => 2,
        }
    }

    const fn percent(self) -> u8 {
        match self {
            Self::ThirtyTwo => 32,
            Self::TwentyFour => 24,
            Self::Forty => 40,
        }
    }
}

impl EditorShare {
    const fn index(self) -> usize {
        match self {
            Self::Forty => 0,
            Self::TwentyFive => 1,
            Self::Sixty => 2,
        }
    }

    const fn percent(self) -> u8 {
        match self {
            Self::Forty => 40,
            Self::TwentyFive => 25,
            Self::Sixty => 60,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Environment {
    Production,
    Staging,
    Development,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Settings {
    allow_new: bool,
    close_reveal: CloseReveal,
    read_only: bool,
    result: ResultFixture,
    environment: Environment,
    editor_share: EditorShare,
    show_explorer: bool,
    explorer_share: ExplorerShare,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Change {
    AllowNew,
    CloseReveal,
    ReadOnly,
    Result(ResultFixture),
    Environment,
    EditorShare,
    ShowExplorer(bool),
    ExplorerShare,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            allow_new: true,
            close_reveal: CloseReveal::Always,
            read_only: false,
            result: ResultFixture::Run,
            environment: Environment::Production,
            editor_share: EditorShare::Forty,
            show_explorer: true,
            explorer_share: ExplorerShare::ThirtyTwo,
        }
    }
}

impl Settings {
    pub(super) const fn allow_new(&self) -> bool {
        self.allow_new
    }

    pub(super) const fn read_only(&self) -> bool {
        self.read_only
    }

    pub(super) const fn editor_share(&self) -> u8 {
        self.editor_share.percent()
    }

    pub(super) const fn show_explorer(&self) -> bool {
        self.show_explorer
    }

    pub(super) const fn explorer_share(&self) -> u8 {
        self.explorer_share.percent()
    }

    pub(super) const fn environment_status(&self) -> (&'static str, Role) {
        match self.environment {
            Environment::Production => ("Production: sample data only", Role::Danger),
            Environment::Staging => ("Staging: sample data only", Role::Warning),
            Environment::Development => ("Development: sample data only", Role::Info),
            Environment::Local => ("Local: sample data only", Role::Fg(FgStep::Muted)),
        }
    }

    pub(super) const fn close_reveal(&self) -> termrock::TabCloseReveal {
        match self.close_reveal {
            CloseReveal::Always => termrock::TabCloseReveal::Always,
            CloseReveal::Hover => termrock::TabCloseReveal::Hover,
        }
    }

    pub(super) fn option_value(&self, key: &str) -> Option<usize> {
        match key {
            ALLOW_NEW_KEY => Some(if self.allow_new { 1 } else { 0 }),
            CLOSE_REVEAL_KEY => Some(match self.close_reveal {
                CloseReveal::Always => 0,
                CloseReveal::Hover => 1,
            }),
            READ_ONLY_KEY => Some(if self.read_only { 1 } else { 0 }),
            RESULT_KEY => Some(match self.result {
                ResultFixture::Run => 0,
                ResultFixture::Empty => 1,
                ResultFixture::Error => 2,
                ResultFixture::Plan => 3,
            }),
            ENVIRONMENT_KEY => Some(match self.environment {
                Environment::Production => 0,
                Environment::Staging => 1,
                Environment::Development => 2,
                Environment::Local => 3,
            }),
            EDITOR_SHARE_KEY => Some(self.editor_share.index()),
            SHOW_EXPLORER_KEY => Some(if self.show_explorer { 1 } else { 0 }),
            EXPLORER_SHARE_KEY => Some(self.explorer_share.index()),
            _ => None,
        }
    }

    pub(super) fn set_option(&mut self, key: &str, value: usize) -> Option<Change> {
        let change = match key {
            ALLOW_NEW_KEY if value <= 1 => {
                let next = value == 1;
                if self.allow_new == next {
                    return None;
                }
                self.allow_new = next;
                Change::AllowNew
            }
            CLOSE_REVEAL_KEY => {
                let next = match value {
                    0 => CloseReveal::Always,
                    1 => CloseReveal::Hover,
                    _ => return None,
                };
                if self.close_reveal == next {
                    return None;
                }
                self.close_reveal = next;
                Change::CloseReveal
            }
            READ_ONLY_KEY if value <= 1 => {
                let next = value == 1;
                if self.read_only == next {
                    return None;
                }
                self.read_only = next;
                Change::ReadOnly
            }
            RESULT_KEY => {
                let next = match value {
                    0 => ResultFixture::Run,
                    1 => ResultFixture::Empty,
                    2 => ResultFixture::Error,
                    3 => ResultFixture::Plan,
                    _ => return None,
                };
                if self.result == next {
                    return None;
                }
                self.result = next;
                Change::Result(next)
            }
            ENVIRONMENT_KEY => {
                let next = match value {
                    0 => Environment::Production,
                    1 => Environment::Staging,
                    2 => Environment::Development,
                    3 => Environment::Local,
                    _ => return None,
                };
                if self.environment == next {
                    return None;
                }
                self.environment = next;
                Change::Environment
            }
            EDITOR_SHARE_KEY => {
                let next = match value {
                    0 => EditorShare::Forty,
                    1 => EditorShare::TwentyFive,
                    2 => EditorShare::Sixty,
                    _ => return None,
                };
                if self.editor_share == next {
                    return None;
                }
                self.editor_share = next;
                Change::EditorShare
            }
            SHOW_EXPLORER_KEY if value <= 1 => {
                let next = value == 1;
                if self.show_explorer == next {
                    return None;
                }
                self.show_explorer = next;
                Change::ShowExplorer(next)
            }
            EXPLORER_SHARE_KEY => {
                let next = match value {
                    0 => ExplorerShare::ThirtyTwo,
                    1 => ExplorerShare::TwentyFour,
                    2 => ExplorerShare::Forty,
                    _ => return None,
                };
                if self.explorer_share == next {
                    return None;
                }
                self.explorer_share = next;
                Change::ExplorerShare
            }
            _ => return None,
        };
        Some(change)
    }

    pub(super) fn dialog_data(&self) -> DialogData {
        DialogData {
            allow_new: self.allow_new,
            close_reveal: self.option_value(CLOSE_REVEAL_KEY).unwrap_or(0),
            read_only: self.read_only,
            result: self.option_value(RESULT_KEY).unwrap_or(0),
            environment: self.option_value(ENVIRONMENT_KEY).unwrap_or(0),
            editor_share: self.option_value(EDITOR_SHARE_KEY).unwrap_or(0),
            show_explorer: self.show_explorer,
            explorer_share: self.option_value(EXPLORER_SHARE_KEY).unwrap_or(0),
            environment_rows: ENVIRONMENT_ROWS,
        }
    }
}

pub(super) struct DialogData {
    allow_new: bool,
    close_reveal: usize,
    read_only: bool,
    result: usize,
    environment: usize,
    editor_share: usize,
    show_explorer: bool,
    explorer_share: usize,
    environment_rows: [(&'static str, Role); 5],
}

impl DialogData {
    pub(super) fn option_for(&self, id: Id) -> Option<(&'static str, usize)> {
        match id {
            ALLOW_NEW => Some((ALLOW_NEW_KEY, if self.allow_new { 1 } else { 0 })),
            CLOSE_REVEAL => Some((CLOSE_REVEAL_KEY, self.close_reveal)),
            READ_ONLY => Some((READ_ONLY_KEY, if self.read_only { 1 } else { 0 })),
            RESULT => Some((RESULT_KEY, self.result)),
            ENVIRONMENT => Some((ENVIRONMENT_KEY, self.environment)),
            EDITOR_SHARE => Some((EDITOR_SHARE_KEY, self.editor_share)),
            SHOW_EXPLORER => Some((SHOW_EXPLORER_KEY, if self.show_explorer { 1 } else { 0 })),
            EXPLORER_SHARE => Some((EXPLORER_SHARE_KEY, self.explorer_share)),
            _ => None,
        }
    }
}

impl FormData for DialogData {
    fn value(&self, id: Id) -> FieldRef<'_> {
        match id {
            ALLOW_NEW => FieldRef::Flag(self.allow_new),
            CLOSE_REVEAL => FieldRef::Choice(self.close_reveal),
            READ_ONLY => FieldRef::Flag(self.read_only),
            RESULT => FieldRef::Choice(self.result),
            ENVIRONMENT => FieldRef::Choice(self.environment),
            ENVIRONMENT_NOTE => FieldRef::Note(&self.environment_rows),
            EDITOR_SHARE => FieldRef::Choice(self.editor_share),
            SHOW_EXPLORER => FieldRef::Flag(self.show_explorer),
            EXPLORER_SHARE => FieldRef::Choice(self.explorer_share),
            _ => FieldRef::Text(""),
        }
    }

    fn value_mut(&mut self, id: Id) -> FieldMut<'_> {
        match id {
            ALLOW_NEW => FieldMut::Flag(&mut self.allow_new),
            CLOSE_REVEAL => FieldMut::Choice(&mut self.close_reveal),
            READ_ONLY => FieldMut::Flag(&mut self.read_only),
            RESULT => FieldMut::Choice(&mut self.result),
            ENVIRONMENT => FieldMut::Choice(&mut self.environment),
            EDITOR_SHARE => FieldMut::Choice(&mut self.editor_share),
            SHOW_EXPLORER => FieldMut::Flag(&mut self.show_explorer),
            EXPLORER_SHARE => FieldMut::Choice(&mut self.explorer_share),
            _ => FieldMut::ReadOnly,
        }
    }

    fn options(&self, id: Id) -> &[&str] {
        match id {
            CLOSE_REVEAL => CLOSE_REVEAL_OPTIONS,
            RESULT => RESULT_OPTIONS,
            ENVIRONMENT => ENVIRONMENT_OPTIONS,
            EDITOR_SHARE => EDITOR_SHARE_OPTIONS,
            EXPLORER_SHARE => EXPLORER_SHARE_OPTIONS,
            _ => &[],
        }
    }

    fn value_and_options(&mut self, id: Id) -> (FieldMut<'_>, &[&str]) {
        let options = match id {
            CLOSE_REVEAL => CLOSE_REVEAL_OPTIONS,
            RESULT => RESULT_OPTIONS,
            ENVIRONMENT => ENVIRONMENT_OPTIONS,
            EDITOR_SHARE => EDITOR_SHARE_OPTIONS,
            EXPLORER_SHARE => EXPLORER_SHARE_OPTIONS,
            _ => &[],
        };
        (self.value_mut(id), options)
    }
}

pub(super) fn dialog() -> FormDialog<'static> {
    FormDialog::new(DIALOG_ID, "Query workbench options", FIELDS)
        .actions(ACTIONS)
        .cancel_action(CLOSE)
        .enter_policy(EnterPolicy::Never)
        .field_sizing(FieldSizingPolicy::Intrinsic)
        .submit_policy(FormDialogSubmitPolicy::KeepOpen)
        .width(58)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn historical_option_keys_keep_the_exact_defaults_and_reject_invalid_values() {
        let mut settings = Settings::default();
        assert_eq!(settings.option_value(ALLOW_NEW_KEY), Some(1));
        assert_eq!(settings.option_value(CLOSE_REVEAL_KEY), Some(0));
        assert_eq!(settings.option_value(READ_ONLY_KEY), Some(0));
        assert_eq!(settings.option_value(RESULT_KEY), Some(0));
        assert_eq!(settings.option_value(ENVIRONMENT_KEY), Some(0));
        assert_eq!(settings.option_value(EDITOR_SHARE_KEY), Some(0));
        assert_eq!(settings.option_value(SHOW_EXPLORER_KEY), Some(1));
        assert_eq!(settings.option_value(EXPLORER_SHARE_KEY), Some(0));
        assert_eq!(EDITOR_SHARE_OPTIONS, &["40", "25", "60"]);
        assert_eq!(EXPLORER_SHARE_OPTIONS, &["32", "24", "40"]);
        assert_eq!(settings.option_value("queryworkbench.explorer"), Some(1));
        assert_eq!(
            settings.option_value("queryworkbench.explorer_width"),
            Some(0)
        );
        assert_eq!(settings.option_value("queryworkbench.show_explorer"), None);
        assert_eq!(settings.option_value("queryworkbench.explorer_share"), None);
        assert_eq!(
            settings.dialog_data().option_for(EDITOR_SHARE),
            Some((EDITOR_SHARE_KEY, 0))
        );

        let defaults = settings.clone();
        for (key, value) in [
            (ALLOW_NEW_KEY, 1),
            (ALLOW_NEW_KEY, 2),
            (CLOSE_REVEAL_KEY, 0),
            (CLOSE_REVEAL_KEY, 2),
            (READ_ONLY_KEY, 0),
            (READ_ONLY_KEY, 2),
            (RESULT_KEY, 0),
            (RESULT_KEY, 4),
            (ENVIRONMENT_KEY, 0),
            (ENVIRONMENT_KEY, 4),
            (EDITOR_SHARE_KEY, 0),
            (EDITOR_SHARE_KEY, 3),
            (SHOW_EXPLORER_KEY, 2),
            (EXPLORER_SHARE_KEY, 3),
            ("queryworkbench.show_explorer", 1),
            ("queryworkbench.explorer_share", 0),
        ] {
            assert_eq!(settings.set_option(key, value), None, "{key}={value}");
            assert_eq!(
                settings, defaults,
                "{key}={value} leaves all settings intact"
            );
        }
    }
    #[test]
    fn editor_share_indices_keep_the_historical_presets_and_change_once() {
        let mut settings = Settings::default();
        assert_eq!(settings.editor_share(), 40);
        assert_eq!(settings.set_option(EDITOR_SHARE_KEY, 0), None);
        assert_eq!(
            settings.set_option(EDITOR_SHARE_KEY, 1),
            Some(Change::EditorShare)
        );
        assert_eq!(settings.editor_share(), 25);
        assert_eq!(settings.option_value(EDITOR_SHARE_KEY), Some(1));
        assert_eq!(
            settings.dialog_data().option_for(EDITOR_SHARE),
            Some((EDITOR_SHARE_KEY, 1))
        );
        assert_eq!(settings.set_option(EDITOR_SHARE_KEY, 1), None);
        assert_eq!(
            settings.set_option(EDITOR_SHARE_KEY, 2),
            Some(Change::EditorShare)
        );
        assert_eq!(settings.editor_share(), 60);
        assert_eq!(settings.option_value(EDITOR_SHARE_KEY), Some(2));
        assert_eq!(
            settings.dialog_data().option_for(EDITOR_SHARE),
            Some((EDITOR_SHARE_KEY, 2))
        );
        assert_eq!(settings.set_option(EDITOR_SHARE_KEY, 2), None);
        assert_eq!(
            settings.set_option(EDITOR_SHARE_KEY, 0),
            Some(Change::EditorShare)
        );
        assert_eq!(settings.editor_share(), 40);
        assert_eq!(
            settings.dialog_data().option_for(EDITOR_SHARE),
            Some((EDITOR_SHARE_KEY, 0))
        );
    }
}

#[cfg(test)]
mod explorer_option_tests {
    use super::*;

    #[test]
    fn explorer_options_default_to_visible_and_keep_the_selected_width() {
        let mut settings = Settings::default();
        assert!(settings.show_explorer());
        assert_eq!(settings.option_value(SHOW_EXPLORER_KEY), Some(1));
        assert_eq!(settings.explorer_share(), 32);
        assert_eq!(settings.option_value(EXPLORER_SHARE_KEY), Some(0));

        assert_eq!(
            settings.set_option(SHOW_EXPLORER_KEY, 0),
            Some(Change::ShowExplorer(false))
        );
        assert!(!settings.show_explorer());
        assert_eq!(settings.set_option(SHOW_EXPLORER_KEY, 0), None);
        assert_eq!(
            settings.set_option(EXPLORER_SHARE_KEY, 1),
            Some(Change::ExplorerShare)
        );
        assert_eq!(settings.explorer_share(), 24);
        assert_eq!(settings.option_value(EXPLORER_SHARE_KEY), Some(1));
    }
}
